use std::fmt;
use url::Url;

#[derive(Clone, Debug, Serialize)]
pub struct ProxyOpts {
    pub target: String,
    pub port: u16,
}

impl ProxyOpts {
    pub fn new(target: impl Into<String>) -> ProxyOpts {
        ProxyOpts {
            target: target.into(),
            ..ProxyOpts::default()
        }
    }
    pub fn with_port(mut self, port: u16) -> ProxyOpts {
        self.port = port;
        self
    }
}

impl Default for ProxyOpts {
    fn default() -> Self {
        ProxyOpts {
            target: String::new(),
            port: 8080,
        }
    }
}

pub fn get_host(url: &str) -> Result<String, ConfigError> {
    let parsed = Url::parse(url).map_err(|e| {
        println!("{}", e);
        ConfigError::UrlInvalid
    })?;
    let h = parsed.host().ok_or(ConfigError::UrlInvalidHost)?;
    Ok(h.to_string())
}

#[derive(Debug)]
pub enum ConfigError {
    UrlInvalid,
    UrlInvalidHost,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::UrlInvalid => write!(f, "Invalid URL"),
            ConfigError::UrlInvalidHost => write!(f, "Could not retrieve the host from the URL"),
        }
    }
}
