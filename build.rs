extern crate build_database;
extern crate diesel;

use build_database::{build_database::Migrate, build_log_debug};
use std::{env, process::Command};

fn main() {
    Migrate::run(None).expect("Unable to run migrations");
    let url = env::var("DATABASE_URL").unwrap();
    build_log_debug!("url: {}", url);
    Command::new("diesel")
        .arg("migration")
        .arg("run")
        .arg(format!("--database-url={:?}", url))
        .spawn()
        .unwrap();
}
