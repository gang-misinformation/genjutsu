use std::env;

#[derive(Debug, Clone)]
pub struct GenBackendConfig {
    pub port: u16,
}

impl GenBackendConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::from_path("crates/gj-app/.env")?;

        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .expect("PORT must be a number");

        Ok(Self {
            port
        })
    }
}