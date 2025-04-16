use std::{ffi::CStr, path::PathBuf, sync::LazyLock};

use anyhow::Result;
use dashmap::DashMap;
use hf_hub::api::sync::Api;

use crate::{
    datatype::{SparsevecOutput, SparsevecOwned},
    model::{load_dynamic_model, ModelPtr},
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

type ModelObjectPool = DashMap<String, ModelPtr>;
static TOKENIZER_OBJECT_POOL: LazyLock<ModelObjectPool> = LazyLock::new(ModelObjectPool::new);

fn get_model(model: &str) -> Result<ModelPtr> {
    match TOKENIZER_OBJECT_POOL.get(model) {
        Some(ptr) => Ok(ptr.clone()),
        None => {
            let model_path = ASSETS_DIR.join(model);
            if !model_path.exists() {
                return Err(anyhow::anyhow!("Model {} not found", model));
            }
            let ptr = load_dynamic_model(&model_path)?;
            TOKENIZER_OBJECT_POOL.insert(model.to_string(), ptr.clone());
            Ok(ptr)
        }
    }
}

pub fn init() {
    // Preload models
    let models = crate::guc::preload_models();
    for model in models {
        if let Err(e) = get_model(&model) {
            pgrx::warning!("Failed to load model {}: {}", model, e);
        }
    }
}

#[pgrx::pg_extern(immutable, strict, parallel_safe)]
fn encode_document(document: &str, model: &str) -> Result<SparsevecOutput> {
    let model = get_model(model)?;
    let tensor = model.encode_document(document)?;
    let vec = tensor.to_vec1::<f32>()?;
    let sparse_vec = SparsevecOwned::from_dense(&vec)?;
    let output = SparsevecOutput::new(sparse_vec.as_borrowed());
    Ok(output)
}

#[pgrx::pg_extern(immutable, strict, parallel_safe)]
fn encode_query(query: &str, model: &str) -> Result<SparsevecOutput> {
    let model = get_model(model)?;
    let tensor = model.encode_query(query)?;
    let vec = tensor.to_vec1::<f32>()?;
    let sparse_vec = SparsevecOwned::from_dense(&vec)?;
    let output = SparsevecOutput::new(sparse_vec.as_borrowed());
    Ok(output)
}

#[pgrx::pg_extern(volatile, strict)]
fn download_model(name: &str, repo_id: String) -> Result<()> {
    use ureq::Error;

    let assets_dir = ASSETS_DIR.join(name);
    if assets_dir.exists() {
        return Err(anyhow::anyhow!("Model {} already exists", name));
    }
    std::fs::create_dir_all(&assets_dir)?;

    let api = Api::new()?;
    let repo = api.model(repo_id);

    let inner = || -> Result<()> {
        for file in ["config.json", "idf.json", "tokenizer.json"] {
            let file_path = assets_dir.join(file);
            let file_url = repo.url(file);
            let mut res = ureq::get(&file_url).call()?;
            let mut reader = res.body_mut().as_reader();
            let mut file = std::fs::File::create(file_path)?;
            std::io::copy(&mut reader, &mut file)?;
        }
        let mut success = false;
        for try_file in ["model.safetensors", "pytorch_model.bin"] {
            let file_path = assets_dir.join(try_file);
            let file_url = repo.url(try_file);
            let mut res = ureq::get(&file_url).call();
            let mut reader = match &mut res {
                Ok(res) => res.body_mut().as_reader(),
                Err(Error::StatusCode(404)) => {
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to download model file {}: {}",
                        try_file,
                        e
                    ));
                }
            };
            std::io::copy(&mut reader, &mut std::fs::File::create(file_path)?)?;
            success = true;
            break;
        }
        if !success {
            return Err(anyhow::anyhow!("No model file found"));
        }

        Ok(())
    };

    match inner() {
        Ok(_) => Ok(()),
        Err(e) => {
            std::fs::remove_dir_all(&assets_dir)?;
            Err(e)
        }
    }
}

#[pgrx::pg_extern(volatile, strict)]
fn delete_model(name: &str) -> Result<()> {
    let assets_dir = ASSETS_DIR.join(name);
    if !assets_dir.exists() {
        return Err(anyhow::anyhow!("Model {} does not exist", name));
    }
    std::fs::remove_dir_all(&assets_dir)?;
    Ok(())
}

#[pgrx::pg_extern(volatile, strict)]
fn list_model() -> Vec<String> {
    let mut models = vec![];
    if let Ok(entries) = std::fs::read_dir(&*ASSETS_DIR) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    models.push(name.to_string());
                }
            }
        }
    }
    models
}
