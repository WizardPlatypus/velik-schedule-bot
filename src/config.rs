#[derive(serde::Deserialize)]
pub struct Config {
    pub token: String,
}

pub fn get() -> Config {
    use config::File;

    let config = config::Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()
        .expect("Failed to build config::Config");

    config.try_deserialize().expect("Invalid configuration")
}
