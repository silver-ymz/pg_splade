::pgrx::pg_module_magic!();

pub mod datatype;
pub mod encode;
pub mod model;

#[cfg(not(all(target_endian = "little", target_pointer_width = "64")))]
compile_error!("Target is not supported.");

#[cfg(not(any(feature = "pg14", feature = "pg15", feature = "pg16", feature = "pg17")))]
compiler_error!("PostgreSQL version must be selected.");

#[pgrx::pg_guard]
unsafe extern "C" fn _PG_init() {
    encode::init();
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
