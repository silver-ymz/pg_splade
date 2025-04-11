use std::{
    ffi::CStr,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Result;
use candle_core::Tensor;
use candle_transformers::models::{bert::BertForMaskedLM, distilbert::DistilBertForMaskedLM};

use crate::{
    datatype::{SparsevecOutput, SparsevecOwned},
    model::{MaskedLM, SpladeModel},
};

static ASSETS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut sharepath = [0u8; pgrx::pg_sys::MAXPGPATH as usize];
    unsafe {
        #[allow(static_mut_refs)]
        pgrx::pg_sys::get_share_path(
            pgrx::pg_sys::my_exec_path.as_ptr(),
            sharepath.as_mut_ptr().cast(),
        )
    };
    let sharepath = CStr::from_bytes_until_nul(&sharepath).unwrap();
    let sharepath = sharepath.to_str().unwrap();
    let mut res = PathBuf::from(sharepath);
    res.push("splade");
    res
});

// opensearch-neural-sparse-encoding-doc-v2-mini
static MINI: LazyLock<SpladeModel<BertForMaskedLM>> = LazyLock::new(|| {
    let path = Path::new(&*ASSETS_DIR).join("mini");
    SpladeModel::load(&path).expect("Failed to load model")
});

// opensearch-neural-sparse-encoding-doc-v3-distill
static DISTILL: LazyLock<SpladeModel<DistilBertForMaskedLM>> = LazyLock::new(|| {
    let path = Path::new(&*ASSETS_DIR).join("distill");
    SpladeModel::load(&path).expect("Failed to load model")
});

trait Encode {
    fn encode_document(&self, document: &str) -> Result<Tensor>;
    fn encode_query(&self, query: &str) -> Result<Tensor>;
}

impl<T: MaskedLM> Encode for SpladeModel<T> {
    fn encode_document(&self, document: &str) -> Result<Tensor> {
        self.encode_document(document)
    }

    fn encode_query(&self, query: &str) -> Result<Tensor> {
        self.encode_query(query)
    }
}

fn get_model(model: &str) -> &'static dyn Encode {
    match model {
        "mini" => &*MINI,
        "distill" => &*DISTILL,
        _ => panic!("Unknown model: {}", model),
    }
}

pub fn init() {
    // TODO: preload model
}

#[pgrx::pg_extern(immutable, strict, parallel_safe)]
fn encode_document(document: &str, model: &str) -> Result<SparsevecOutput> {
    let model = get_model(model);
    let tensor = model.encode_document(document)?;
    let vec = tensor.to_vec1::<f32>()?;
    let sparse_vec = SparsevecOwned::from_dense(&vec)?;
    let output = SparsevecOutput::new(sparse_vec.as_borrowed());
    Ok(output)
}

#[pgrx::pg_extern(immutable, strict, parallel_safe)]
fn encode_query(query: &str, model: &str) -> Result<SparsevecOutput> {
    let model = get_model(model);
    let tensor = model.encode_query(query)?;
    let vec = tensor.to_vec1::<f32>()?;
    let sparse_vec = SparsevecOwned::from_dense(&vec)?;
    let output = SparsevecOutput::new(sparse_vec.as_borrowed());
    Ok(output)
}
