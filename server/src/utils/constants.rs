use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env;
use std::error::Error;

lazy_static! {
    pub static ref DATABASE_URL: String = set_db_url().expect("Failed to get DATABASE_URL");
    pub static ref SECRET: String = set_secret().expect("Failed to get SECRET");
}

fn set_db_url() -> Result<String, env::VarError> {
    dotenv().ok();
    match env::var("DATABASE_URL") {
        Ok(url) => {
            println!("DATABASE_URL: {}", url); // Log the DATABASE_URL
            Ok(url)
        }
        Err(e) => {
            println!("Failed to get DATABASE_URL: {}", e); // Log the error
            Err(e)
        }
    }
}

fn set_secret() -> Result<String, env::VarError> {
    dotenv().ok();
    match env::var("SECRET") {
        Ok(secret) => {
            println!("SECRET: {}", secret); // Log the SECRET
            Ok(secret)
        }
        Err(e) => {
            println!("Failed to get SECRET: {}", e); // Log the error
            Err(e)
        }
    }
}
