#[cfg(feature = "database")]
extern crate diesel;
pub mod build_database;
pub mod migrations;
#[macro_use]
#[cfg(feature = "diesel_migrations")]
extern crate diesel_migrations;
pub mod error;
// pub mod keygen;

#[macro_export]
macro_rules! build_log_info {
    ($($tokens: tt)*) => {
        println!("cargo:warning=INFO - {}", format!($($tokens)*))
    }
}

#[macro_export]
macro_rules! build_log_debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning=DEBUG - {}", format!($($tokens)*))
    }
}
