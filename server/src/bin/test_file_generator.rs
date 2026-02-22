use std::error::Error;

use zupeload::{Config, logger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::init()?;

    logger::init(&config.rust_log);

    println!("Hello, world!");

    Ok(())
}
