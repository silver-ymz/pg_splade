use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Error, Result};
use candle_core::{
    utils::{cuda_is_available, metal_is_available},
    DType, Device, Tensor,
};
use candle_nn::VarBuilder;
use candle_transformers::models::{bert::BertForMaskedLM, distilbert::DistilBertForMaskedLM};
use tokenizers::{PaddingParams, Tokenizer};

pub trait MaskedLM {
    type Config: for<'de> serde::Deserialize<'de>;
    const DTYPE: DType;

    fn load(vb: VarBuilder, config: &Self::Config) -> Result<Self>
    where
        Self: Sized;

    fn forward(&self, input_ids: &Tensor, attention_mask: &Tensor) -> Result<Tensor>;

    fn activation(vector: &Tensor) -> Result<Tensor>;
}

pub struct SpladeModel<T> {
    model: T,
    tokenizer: Tokenizer,
    idf: Tensor,
    special_token_id_mask: Tensor,
    device: Device,
    vocab_size: usize,
}

impl<T: MaskedLM> SpladeModel<T> {
    pub fn load(path: &Path) -> Result<SpladeModel<T>> {
        load_model::<T>(path)
    }

    pub fn encode_document(&self, document: &str) -> Result<Tensor> {
        let feature = self
            .tokenizer
            .encode_fast(document, true)
            .map_err(Error::msg)?;
        let input_ids = Tensor::new(feature.get_ids(), &self.device)?.unsqueeze(0)?;
        let attention_mask =
            Tensor::new(feature.get_attention_mask(), &self.device)?.unsqueeze(0)?;
        let ys = self.model.forward(&input_ids, &attention_mask)?;

        let vector = ys
            .broadcast_mul(&attention_mask.unsqueeze(2)?.to_dtype(T::DTYPE)?)?
            .max(1)?;
        let vector = T::activation(&vector)?;
        let vector = vector.broadcast_mul(&self.special_token_id_mask)?;
        let vector = vector.squeeze(0)?;
        Ok(vector)
    }

    pub fn encode_query(&self, query: &str) -> Result<Tensor> {
        let feature = self
            .tokenizer
            .encode_fast(query, true)
            .map_err(Error::msg)?;
        let input_ids = feature.get_ids();

        let mut query_vector = vec![0.0f32; self.vocab_size];
        for id in input_ids {
            query_vector[*id as usize] = 1.0;
        }
        let query_tensor = Tensor::from_vec(query_vector, self.vocab_size, &self.device)?;

        let res = query_tensor.broadcast_mul(&self.idf)?;
        Ok(res)
    }
}

struct LoadContext {
    assets_path: PathBuf,
    device: Device,
    dtype: DType,
}

fn load_model<T: MaskedLM>(path: &Path) -> Result<SpladeModel<T>> {
    let ctx = LoadContext {
        assets_path: path.to_path_buf(),
        device: device()?,
        dtype: T::DTYPE,
    };

    let config = std::fs::read_to_string(ctx.assets_path.join("config.json"))?;
    let config: T::Config = serde_json::from_str(&config)?;
    let mut tokenizer =
        Tokenizer::from_file(ctx.assets_path.join("tokenizer.json")).map_err(Error::msg)?;
    if let Some(pp) = tokenizer.get_padding_mut() {
        pp.strategy = tokenizers::PaddingStrategy::BatchLongest;
    } else {
        tokenizer.with_padding(Some(PaddingParams {
            strategy: tokenizers::PaddingStrategy::BatchLongest,
            ..Default::default()
        }));
    }
    if let Some(tr) = tokenizer.get_truncation_mut() {
        tr.strategy = tokenizers::TruncationStrategy::LongestFirst;
    } else {
        tokenizer
            .with_truncation(Some(tokenizers::TruncationParams {
                strategy: tokenizers::TruncationStrategy::LongestFirst,
                ..Default::default()
            }))
            .map_err(Error::msg)?;
    }

    let vb = if ctx.assets_path.join("pytorch_model.bin").exists() {
        VarBuilder::from_pth(
            ctx.assets_path.join("pytorch_model.bin"),
            ctx.dtype,
            &ctx.device,
        )?
    } else {
        unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[ctx.assets_path.join("model.safetensors")],
                ctx.dtype,
                &ctx.device,
            )
        }?
    };
    let model = T::load(vb, &config)?;
    let idf = get_tokenizer_idf(&tokenizer, &ctx)?;

    let mut special_token_id_vec = vec![1.0f32; tokenizer.get_vocab_size(true) as usize];
    for (k, v) in tokenizer.get_added_vocabulary().get_vocab() {
        if tokenizer.get_added_vocabulary().is_special_token(k) {
            special_token_id_vec[*v as usize] = 0.0;
        }
    }
    let special_token_id_mask = Tensor::from_vec(
        special_token_id_vec,
        tokenizer.get_vocab_size(true) as usize,
        &ctx.device,
    )?;

    let vocab_size = tokenizer.get_vocab_size(true) as usize;

    Ok(SpladeModel {
        model,
        tokenizer,
        idf,
        special_token_id_mask,
        device: ctx.device,
        vocab_size,
    })
}

fn device() -> Result<Device> {
    let res = if cuda_is_available() {
        Device::new_cuda(0)?
    } else if metal_is_available() {
        Device::new_metal(0)?
    } else {
        Device::Cpu
    };
    Ok(res)
}

fn get_tokenizer_idf(tokenizer: &Tokenizer, ctx: &LoadContext) -> Result<Tensor> {
    let idf_content = std::fs::read(ctx.assets_path.join("idf.json"))?;
    let idf: HashMap<String, f32> = serde_json::from_slice(&idf_content)?;

    let mut idf_tensor = vec![0.0; tokenizer.get_vocab_size(true)];
    for (token, weight) in idf {
        let id = tokenizer
            .token_to_id(&token)
            .ok_or(anyhow!("Token not found"))?;
        idf_tensor[id as usize] = weight;
    }
    let res = Tensor::from_vec(
        idf_tensor,
        tokenizer.get_vocab_size(true),
        &ctx.device,
    )?;
    Ok(res)
}

impl MaskedLM for BertForMaskedLM {
    type Config = candle_transformers::models::bert::Config;
    const DTYPE: DType = candle_transformers::models::bert::DTYPE;

    fn load(vb: VarBuilder, config: &Self::Config) -> Result<Self> {
        BertForMaskedLM::load(vb, config).map_err(Error::msg)
    }

    fn forward(&self, input_ids: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        let token_type_ids = input_ids.zeros_like()?;
        self.forward(input_ids, &token_type_ids, Some(attention_mask))
            .map_err(Error::msg)
    }

    fn activation(vector: &Tensor) -> Result<Tensor> {
        let one = Tensor::ones_like(vector)?;
        let vector = vector.relu()?;
        let vector = Tensor::log(&one.broadcast_add(&vector)?)?;
        Ok(vector)
    }
}

impl MaskedLM for DistilBertForMaskedLM {
    type Config = candle_transformers::models::distilbert::Config;
    const DTYPE: DType = candle_transformers::models::distilbert::DTYPE;

    fn load(vb: VarBuilder, config: &Self::Config) -> Result<Self> {
        DistilBertForMaskedLM::load(vb, config).map_err(Error::msg)
    }

    fn forward(&self, input_ids: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        self.forward(input_ids, attention_mask).map_err(Error::msg)
    }

    fn activation(vector: &Tensor) -> Result<Tensor> {
        let one = Tensor::ones_like(vector)?;
        let vector = vector.relu()?;
        let vector = Tensor::log(&one.broadcast_add(&vector)?)?;
        let vector = Tensor::log(&one.broadcast_add(&vector)?)?;
        Ok(vector)
    }
}
