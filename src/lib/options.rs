use std::fmt;
use url::Url;

#[derive(Clone, Debug, Serialize)]
pub enum ProxyScheme {
    Http,
    Https,
}

impl std::fmt::Display for ProxyScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ProxyScheme::Http => write!(f, "http"),
            ProxyScheme::Https => write!(f, "https"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProxyOpts {
    pub target: String,
    pub scheme: ProxyScheme,
    pub port: u16,
}

impl ProxyOpts {
    pub fn new(target: impl Into<String>, scheme: impl Into<String>) -> ProxyOpts {
        ProxyOpts {
            target: target.into(),
            scheme: match scheme.into().as_str() {
                "http" => ProxyScheme::Http,
                "https" => ProxyScheme::Https,
                _ => ProxyScheme::Http,
            },
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
            scheme: ProxyScheme::Http,
            port: 8080,
        }
    }
}

pub fn get_host(url: &str) -> Result<(String, String), ConfigError> {
    let parsed = Url::parse(url).map_err(|e| {
        eprintln!("{}", e);
        ConfigError::UrlInvalid
    })?;
    let host = parsed.host().ok_or(ConfigError::UrlInvalidHost)?;
    let scheme = parsed.scheme();
    Ok((host.to_string(), scheme.to_string()))
}

#[derive(Debug)]
pub enum ConfigError {
    UrlInvalid,
    UrlInvalidHost,
    UrlInvalidScheme,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::UrlInvalid => write!(f, "Invalid URL"),
            ConfigError::UrlInvalidHost => write!(f, "Could not retrieve the host from the URL"),
            ConfigError::UrlInvalidScheme => {
                write!(f, "Could not retrieve the scheme from the URL")
            }
        }
    }
}
