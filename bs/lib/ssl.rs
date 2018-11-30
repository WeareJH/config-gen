use config::ProgramStartError;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use std::fs::File;
use std::io::{Error, Write};
use std::path::PathBuf;
use tempdir::TempDir;

const TMP_DIR_NAME: &'static str = "config-gen";

const TMP_KEY: &'static [u8] = include_bytes!("./key.pem");
const TMP_KEY_NAME: &'static str = "key.pem";

const TMP_CERT: &'static [u8] = include_bytes!("./cert.pem");
const TMP_CERT_NAME: &'static str = "cert.pem";

///
/// Create an SslAcceptorBuilder by using self-signed
/// certificates that exist inside this binary
///
/// This is acceptable since this is a development only
/// tool and nothing this runs should be anywhere near anything
/// that's shared, or in production.
///
pub fn builder() -> Result<SslAcceptorBuilder, ProgramStartError> {
    let (key_path, cert_path, tmp_dir) = ssl_paths().map_err(|_e| ProgramStartError::SslTempDir)?;

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .map_err(|_e| ProgramStartError::SslFailed)?;

    builder
        .set_private_key_file(key_path, SslFiletype::PEM)
        .map_err(|_e| ProgramStartError::SslFailed)?;

    builder
        .set_certificate_chain_file(cert_path)
        .map_err(|_e| ProgramStartError::SslFailed)?;

    tmp_dir
        .close()
        .map_err(|_e| ProgramStartError::SslTempDirClose)?;

    Ok(builder)
}

#[test]
fn test_ssl_builder() {
    builder().unwrap();
}

///
/// Takes the self-signed bundled key & cert
/// and places them in a temporary directory so that they
/// can be used by openSSL
///
/// # Examples
///
/// ```
/// use bs::ssl::*;
/// let (key_path, cert_path, tmp_dir) = ssl_paths().unwrap();
/// println!("key={:?}, cert={:?}", key_path, cert_path);
/// tmp_dir.close().unwrap();
/// ```
///
pub fn ssl_paths() -> Result<(PathBuf, PathBuf, TempDir), Error> {
    let tmp_dir = TempDir::new(TMP_DIR_NAME)?;
    let key_path = tmp_dir.path().join(TMP_KEY_NAME);
    let cert_path = tmp_dir.path().join(TMP_CERT_NAME);

    let mut key_file = File::create(&key_path)?;
    key_file.write_all(TMP_KEY)?;
    key_file.sync_all()?;

    let mut cert_file = File::create(&cert_path)?;
    cert_file.write_all(TMP_CERT)?;
    cert_file.sync_all()?;

    Ok((key_path, cert_path, tmp_dir))
}

#[test]
fn test_ssl_paths() {
    let (_file_key, _file_cert, tmp_dir) = ssl_paths().unwrap();
    assert_eq!(tmp_dir.path().exists(), true);
}
