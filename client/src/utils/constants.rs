use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env;
use std::error::Error;

lazy_static! {
    pub static ref SERVER_ADDR: String = set_serv_addr().expect("Failed to get SERVER_ADDR");
}

fn set_serv_addr() -> Result<String, env::VarError> {
    dotenv().ok();
    match env::var("SERVER_ADDR") {
        Ok(url) => {
            Ok(url)
        }
        Err(e) => {
            println!("Failed to get SERVER_ADDR: {}", e); // Log the error
            Err(e)
        }
    }
}
