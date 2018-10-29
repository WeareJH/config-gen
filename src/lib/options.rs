use clap::App as ClapApp;
use clap::Arg;
use clap::ArgMatches;
use clap::Error;
use config::ProgramStartError;
use std;
use std::ffi::OsString;
use std::fmt;
use url::ParseError;
use url::Url;

#[derive(Clone, Debug, Serialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ProgramOptions {
    pub target: String,
    pub scheme: ProxyScheme,
    pub port: u16,
    pub config_file: Option<String>,
    pub seed_file: Option<String>,
}

impl ProgramOptions {
    pub fn new(target: impl Into<String>, scheme: impl Into<String>) -> ProgramOptions {
        ProgramOptions {
            target: target.into(),
            scheme: match scheme.into().as_str() {
                "http" => ProxyScheme::Http,
                "https" => ProxyScheme::Https,
                _ => ProxyScheme::Http,
            },
            ..ProgramOptions::default()
        }
    }
    ///
    /// Allow ProgramOptions to be constructed from any supported
    /// iterable, like a Vec - this removes the env-specific parts of this
    /// logic to enable simpler testing
    ///
    pub fn from_args<I, T>(args: I) -> Result<ProgramOptions, ProgramStartError>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = ClapApp::new("bs-rust")
            .arg(Arg::with_name("url").required(true))
            .arg(
                Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .takes_value(true),
            ).arg(
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .takes_value(true),
            ).arg(Arg::with_name("seed").long("seed").takes_value(true))
            .get_matches_from_safe(args);
        ProgramOptions::from_matches(matches)
    }

    pub fn from_matches(
        _matches: Result<ArgMatches, Error>,
    ) -> Result<ProgramOptions, ProgramStartError> {
        let matches = _matches.map_err(|e| ProgramStartError::InvalidArgs(e))?;

        let (host, scheme) = get_host(matches.value_of("url").expect("input is required"))
            .map_err(|e| ProgramStartError::ConfigCliError(e))?;

        let port: u16 = matches
            .value_of("port")
            .unwrap_or("0")
            .parse()
            .map_err(|_e| ProgramStartError::ConfigCliError(ConfigError::UrlInvalidPort))?;

        let outgoing_opts = ProgramOptions::new(host, scheme)
            .with_port(port)
            .with_seed_file(matches.value_of("seed"));

        let outgoing_opts = match matches.value_of("config") {
            Some(cfg_file) => outgoing_opts.with_config_file(cfg_file),
            None => outgoing_opts,
        };

        Ok(outgoing_opts)
    }
    pub fn with_port(mut self, port: u16) -> ProgramOptions {
        self.port = port;
        self
    }
    pub fn with_config_file(mut self, path: &str) -> ProgramOptions {
        self.config_file = Some(path.into());
        self
    }
    pub fn with_seed_file(mut self, path: Option<&str>) -> ProgramOptions {
        path.map(|p| {
            self.seed_file = Some(p.into());
        });
        self
    }
}

impl Default for ProgramOptions {
    fn default() -> Self {
        ProgramOptions {
            target: String::new(),
            scheme: ProxyScheme::Http,
            port: 0,
            config_file: None,
            seed_file: None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    UrlInvalid(ParseError),
    UrlInvalidHost,
    UrlInvalidPort,
    UrlInvalidScheme,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::UrlInvalid(e) => write!(f, "{}", e),
            ConfigError::UrlInvalidHost => write!(f, "Could not retrieve the host from the URL"),
            ConfigError::UrlInvalidPort => write!(
                f,
                "Invalid Port provided. Please provide a number between 1024 & 9999"
            ),
            ConfigError::UrlInvalidScheme => {
                write!(f, "Could not retrieve the scheme from the URL")
            }
        }
    }
}

pub fn get_host(url: &str) -> Result<(String, String), ConfigError> {
    let parsed = Url::parse(url).map_err(|e| ConfigError::UrlInvalid(e))?;
    let host = parsed.host().ok_or(ConfigError::UrlInvalidHost)?;
    let scheme = parsed.scheme();
    Ok((host.to_string(), scheme.to_string()))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_vec() {
        let args = vec![
            "/bin/fake-program",
            "https://example.com",
            "--port",
            "9000",
            "--config",
            "test/fixtures/config.yml",
        ];
        let p = ProgramOptions::from_args(args).unwrap();
        assert_eq!(
            p,
            ProgramOptions {
                target: "example.com".to_string(),
                scheme: ProxyScheme::Https,
                port: 9000,
                config_file: Some("test/fixtures/config.yml".into()),
                seed_file: None,
            }
        );
    }
    #[test]
    fn test_from_vec_invalid_url() {
        let args = vec![
            "/bin/fake-program",
            "examplecom",
            "--config",
            "test/fixtures/config.yml",
        ];
        match ProgramOptions::from_args(args) {
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    "URL parse error: relative URL without a base"
                );
            }
            _ => { /* */ }
        }
    }
    #[test]
    fn test_from_vec_invalid_port() {
        let args = vec![
            "/bin/fake-program",
            "https://example.com",
            "--config",
            "test/fixtures/config.yml",
            "--port",
            "900o",
        ];
        match ProgramOptions::from_args(args) {
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    "Invalid Port provided. Please provide a number between 1024 & 9999"
                );
            }
            _ => { /* */ }
        }
    }
}
