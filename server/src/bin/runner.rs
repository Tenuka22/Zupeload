use zupeload::{config::Config, logger};

fn main() -> Result<(), String> {
    let config = Config::init()?;

    logger::init(&config.rust_log);

    println!("Hello, world!");

    Ok(())
}
