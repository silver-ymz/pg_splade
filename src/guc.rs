use std::ffi::CStr;

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};

pub static PRELOAD_MODELS: GucSetting<Option<&CStr>> = GucSetting::<Option<&CStr>>::new(Some(c""));

pub fn init() {
    GucRegistry::define_string_guc(
        "splade.preload_models",
        "Preload models for Splade extension",
        "Comma-separated list of models to preload. Available models: mini, distill.",
        &PRELOAD_MODELS,
        GucContext::Userset,
        GucFlags::default(),
    );

    unsafe {
        #[cfg(any(feature = "pg13", feature = "pg14"))]
        pgrx::pg_sys::EmitWarningsOnPlaceholders(c"splade".as_ptr());
        #[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17"))]
        pgrx::pg_sys::MarkGUCPrefixReserved(c"splade".as_ptr());
    }
}

pub fn preload_models() -> Vec<String> {
    let models = PRELOAD_MODELS.get().unwrap_or_default();
    if models.is_empty() {
        return vec![];
    }
    models
        .to_str()
        .unwrap()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}
