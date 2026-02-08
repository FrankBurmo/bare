//! Gemini-protokoll klient
//!
//! Implementerer Gemini-protokollen (gemini://) med TOFU (Trust On First Use)
//! sertifikathåndtering. Bruker TLS over TCP på port 1965.

use log::{debug, info, warn};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error as TlsError, SignatureScheme};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use url::Url;

/// Standard Gemini-port
const DEFAULT_PORT: u16 = 1965;

/// Maksimal respons-størrelse (5 MB)
const MAX_RESPONSE_SIZE: usize = 5 * 1024 * 1024;

/// Maksimal URL-lengde i bytes
const MAX_URL_LENGTH: usize = 1024;

/// Maksimalt antall redirects å følge
const MAX_REDIRECTS: u8 = 5;

/// Internt resultat fra en enkelt fetch-operasjon
enum FetchOutcome {
    /// Ferdig resultat
    Success(GeminiResponse),
    /// Redirect til ny URL
    Redirect(String),
}

/// Feil som kan oppstå under Gemini-forespørsler
#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("Ugyldig URL: {0}")]
    InvalidUrl(String),

    #[error("TLS-feil: {0}")]
    TlsError(String),

    #[error("Tilkoblingsfeil: {0}")]
    ConnectionError(String),

    #[error("Timeout: Serveren svarte ikke innen {0} sekunder")]
    Timeout(u64),

    #[error("Respons for stor (over {0} bytes)")]
    TooLarge(usize),

    #[error("Ugyldig respons fra serveren: {0}")]
    InvalidResponse(String),

    #[error("Serveren ber om input: {0}")]
    InputRequired(String),

    #[error("Sensitiv input forespurt: {0}")]
    SensitiveInputRequired(String),

    #[error("For mange redirects (maks {0})")]
    RedirectLoop(u8),

    #[error("Sertifikatet for {host} har endret seg!\nGammelt fingerprint: {old_fp}\nNytt fingerprint: {new_fp}\nDette kan indikere et man-in-the-middle angrep.")]
    CertificateChanged {
        host: String,
        old_fp: String,
        new_fp: String,
    },

    #[error("Serveren krever klientsertifikat. Dette er ikke støttet ennå.")]
    ClientCertRequired,

    #[error("Gemini-feil ({status}): {meta}")]
    ServerError { status: u8, meta: String },
}

/// Respons fra en Gemini-server
#[derive(Debug)]
#[allow(dead_code)]
pub struct GeminiResponse {
    /// Statuskode (første siffer: 1-6)
    pub status: u8,
    /// Meta-felt (MIME-type for 2x, URL for 3x, feilmelding for 4x/5x)
    pub meta: String,
    /// Respons-body (kun for 2x-statuskoder)
    pub body: Option<String>,
    /// Den endelige URL-en (etter eventuelle redirects)
    pub final_url: String,
}

/// Lagret sertifikat for TOFU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCert {
    /// SHA-256 fingerprint av sertifikatet
    pub fingerprint: String,
    /// Når sertifikatet ble sett første gang
    pub first_seen: String,
    /// Når sertifikatet sist ble sett
    pub last_seen: String,
}

/// TOFU (Trust On First Use) sertifikatlagring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TofuStore {
    /// Kjente verter og deres sertifikater (nøkkel: "host:port")
    hosts: HashMap<String, StoredCert>,
}

impl TofuStore {
    /// Last TOFU-lageret fra fil
    pub fn load(path: &PathBuf) -> Self {
        if !path.exists() {
            return Self::default();
        }

        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Lagre TOFU-lageret til fil
    pub fn save(&self, path: &PathBuf) -> Result<(), GeminiError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| GeminiError::TlsError(format!("Kunne ikke opprette mappe: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| GeminiError::TlsError(format!("Serialiseringsfeil: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| GeminiError::TlsError(format!("Kunne ikke skrive fil: {}", e)))?;

        Ok(())
    }

    /// Sjekk et sertifikat mot TOFU-lageret
    ///
    /// Returnerer Ok(true) hvis sertifikatet ble akseptert (nytt eller kjent),
    /// eller en CertificateChanged-feil hvis fingerprint har endret seg.
    pub fn verify(&mut self, host_port: &str, fingerprint: &str) -> Result<bool, GeminiError> {
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(stored) = self.hosts.get_mut(host_port) {
            if stored.fingerprint == fingerprint {
                // Kjent sertifikat — oppdater last_seen
                stored.last_seen = now;
                Ok(true)
            } else {
                // Sertifikatet har endret seg!
                Err(GeminiError::CertificateChanged {
                    host: host_port.to_string(),
                    old_fp: stored.fingerprint.clone(),
                    new_fp: fingerprint.to_string(),
                })
            }
        } else {
            // Ukjent vert — lagre sertifikatet
            info!("TOFU: Lagrer nytt sertifikat for {}", host_port);
            self.hosts.insert(
                host_port.to_string(),
                StoredCert {
                    fingerprint: fingerprint.to_string(),
                    first_seen: now.clone(),
                    last_seen: now,
                },
            );
            Ok(true)
        }
    }
}

/// Beregn SHA-256 fingerprint av et sertifikat
fn cert_fingerprint(cert: &CertificateDer) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cert.as_ref());
    hex::encode(hasher.finalize())
}

/// Custom ServerCertVerifier som aksepterer alle sertifikater.
/// TOFU-sjekken gjøres separat etter TLS-handshake.
#[derive(Debug)]
struct TofuVerifier;

impl ServerCertVerifier for TofuVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        // Aksepterer alle sertifikater — TOFU-sjekk gjøres etter tilkobling
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

/// Gemini-klient med TOFU-sertifikathåndtering
pub struct GeminiClient {
    /// TLS-konfigurasjon
    tls_config: Arc<ClientConfig>,
    /// TOFU sertifikatlagring
    tofu_store: Mutex<TofuStore>,
    /// Sti til TOFU-lagringsfil
    tofu_path: PathBuf,
    /// Timeout i sekunder
    timeout_seconds: u64,
}

impl GeminiClient {
    /// Opprett en ny GeminiClient
    pub fn new() -> Self {
        Self::with_timeout(30)
    }

    /// Opprett en ny GeminiClient med egendefinert timeout
    pub fn with_timeout(timeout_seconds: u64) -> Self {
        let tls_config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(TofuVerifier))
            .with_no_client_auth();

        let tofu_path = get_tofu_path();
        let tofu_store = TofuStore::load(&tofu_path);

        Self {
            tls_config: Arc::new(tls_config),
            tofu_store: Mutex::new(tofu_store),
            tofu_path,
            timeout_seconds,
        }
    }

    /// Valider og parse en Gemini-URL
    pub fn validate_url(url_str: &str) -> Result<Url, GeminiError> {
        let parsed = Url::parse(url_str)
            .map_err(|e| GeminiError::InvalidUrl(format!("{}: {}", url_str, e)))?;

        if parsed.scheme() != "gemini" {
            return Err(GeminiError::InvalidUrl(format!(
                "Forventet gemini://-scheme, fikk {}://",
                parsed.scheme()
            )));
        }

        if parsed.host_str().is_none() {
            return Err(GeminiError::InvalidUrl("URL mangler vertsnavn".to_string()));
        }

        // Sjekk URL-lengde
        if url_str.len() > MAX_URL_LENGTH {
            return Err(GeminiError::InvalidUrl(format!(
                "URL er for lang ({} bytes, maks {})",
                url_str.len(),
                MAX_URL_LENGTH
            )));
        }

        Ok(parsed)
    }

    /// Hent innhold fra en Gemini-URL
    pub async fn fetch(&self, url_str: &str) -> Result<GeminiResponse, GeminiError> {
        let mut current_url = url_str.to_string();
        let mut redirect_count: u8 = 0;

        loop {
            if redirect_count > MAX_REDIRECTS {
                return Err(GeminiError::RedirectLoop(MAX_REDIRECTS));
            }

            match self.fetch_single(&current_url).await? {
                FetchOutcome::Success(response) => return Ok(response),
                FetchOutcome::Redirect(new_url) => {
                    info!(
                        "Gemini: Redirect {} -> {} (#{}/{})",
                        current_url,
                        new_url,
                        redirect_count + 1,
                        MAX_REDIRECTS
                    );
                    current_url = new_url;
                    redirect_count += 1;
                }
            }
        }
    }

    /// Intern fetch for én enkelt forespørsel (uten redirect-følging)
    async fn fetch_single(&self, url_str: &str) -> Result<FetchOutcome, GeminiError> {
        let url = Self::validate_url(url_str)?;
        let host = url
            .host_str()
            .ok_or_else(|| GeminiError::InvalidUrl("Mangler vertsnavn".to_string()))?
            .to_string();
        let port = url.port().unwrap_or(DEFAULT_PORT);
        let host_port = format!("{}:{}", host, port);

        info!("Gemini: Kobler til {}", host_port);

        // TCP-tilkobling med timeout
        let tcp_stream = tokio::time::timeout(
            Duration::from_secs(self.timeout_seconds),
            TcpStream::connect(&host_port),
        )
        .await
        .map_err(|_| GeminiError::Timeout(self.timeout_seconds))?
        .map_err(|e| GeminiError::ConnectionError(e.to_string()))?;

        debug!("Gemini: TCP-tilkobling etablert til {}", host_port);

        // TLS-handshake
        let server_name = ServerName::try_from(host.clone())
            .map_err(|e| GeminiError::TlsError(format!("Ugyldig vertsnavn: {}", e)))?;

        let connector = TlsConnector::from(self.tls_config.clone());

        let tls_stream = tokio::time::timeout(
            Duration::from_secs(self.timeout_seconds),
            connector.connect(server_name, tcp_stream),
        )
        .await
        .map_err(|_| GeminiError::Timeout(self.timeout_seconds))?
        .map_err(|e| GeminiError::TlsError(e.to_string()))?;

        debug!("Gemini: TLS-handshake fullført");

        // TOFU-sjekk: Verifiser serversertifikatet
        let (io, session) = tls_stream.get_ref();
        let _ = io; // Vi trenger kun session for sertifikater

        if let Some(certs) = session.peer_certificates() {
            if let Some(cert) = certs.first() {
                let fingerprint = cert_fingerprint(cert);
                debug!("Gemini: Sertifikat-fingerprint mottatt for {}", host_port);

                let mut store = self.tofu_store.lock().unwrap();
                store.verify(&host_port, &fingerprint)?;

                // Lagre oppdatert TOFU-lager
                if let Err(e) = store.save(&self.tofu_path) {
                    warn!("Kunne ikke lagre TOFU-lager: {}", e);
                }
            }
        }

        // Send forespørsel
        let request = format!("{}\r\n", url.as_str());
        let (mut read_half, mut write_half) = tokio::io::split(tls_stream);

        write_half
            .write_all(request.as_bytes())
            .await
            .map_err(|e| {
                GeminiError::ConnectionError(format!("Kunne ikke sende forespørsel: {}", e))
            })?;

        debug!("Gemini: Forespørsel sendt: {}", url.as_str());

        // Les respons-header
        let mut reader = BufReader::new(&mut read_half);
        let mut header_line = String::new();

        tokio::time::timeout(
            Duration::from_secs(self.timeout_seconds),
            reader.read_line(&mut header_line),
        )
        .await
        .map_err(|_| GeminiError::Timeout(self.timeout_seconds))?
        .map_err(|e| GeminiError::InvalidResponse(format!("Kunne ikke lese header: {}", e)))?;

        debug!("Gemini: Respons-header: {:?}", header_line.trim());

        // Parse status og meta
        let (status, meta) = parse_response_header(&header_line)?;

        // Håndter statuskoder
        match status / 10 {
            1 => {
                // Input påkrevd
                if status == 11 {
                    Err(GeminiError::SensitiveInputRequired(meta))
                } else {
                    Err(GeminiError::InputRequired(meta))
                }
            }
            2 => {
                // Suksess — les body
                let mut body = Vec::new();
                let bytes_read = tokio::time::timeout(
                    Duration::from_secs(self.timeout_seconds),
                    reader.take(MAX_RESPONSE_SIZE as u64).read_to_end(&mut body),
                )
                .await
                .map_err(|_| GeminiError::Timeout(self.timeout_seconds))?
                .map_err(|e| {
                    GeminiError::InvalidResponse(format!("Feil under lesing av body: {}", e))
                })?;

                debug!("Gemini: Mottatt {} bytes body", bytes_read);

                if bytes_read >= MAX_RESPONSE_SIZE {
                    return Err(GeminiError::TooLarge(MAX_RESPONSE_SIZE));
                }

                let body_str = String::from_utf8_lossy(&body).to_string();

                Ok(FetchOutcome::Success(GeminiResponse {
                    status,
                    meta,
                    body: Some(body_str),
                    final_url: url_str.to_string(),
                }))
            }
            3 => {
                // Redirect
                let redirect_url = if meta.starts_with("gemini://") || meta.starts_with("//") {
                    meta.clone()
                } else {
                    // Relativ URL — løs mot nåværende
                    url.join(&meta)
                        .map(|u| u.to_string())
                        .unwrap_or(meta.clone())
                };

                Ok(FetchOutcome::Redirect(redirect_url))
            }
            4 | 5 => {
                // Midlertidig/permanent feil
                Err(GeminiError::ServerError { status, meta })
            }
            6 => {
                // Klientsertifikat påkrevd
                Err(GeminiError::ClientCertRequired)
            }
            _ => Err(GeminiError::InvalidResponse(format!(
                "Ukjent statuskode: {}",
                status
            ))),
        }
    }
}

/// Parse Gemini respons-header
///
/// Format: <STATUS><SPACE><META>\r\n
fn parse_response_header(header: &str) -> Result<(u8, String), GeminiError> {
    let header = header.trim_end_matches('\n').trim_end_matches('\r');

    if header.len() < 2 {
        return Err(GeminiError::InvalidResponse("Header for kort".to_string()));
    }

    let status_str = &header[..2];
    let status: u8 = status_str.parse().map_err(|_| {
        GeminiError::InvalidResponse(format!("Ugyldig statuskode: '{}'", status_str))
    })?;

    // Status må være mellom 10 og 69
    if !(10..=69).contains(&status) {
        return Err(GeminiError::InvalidResponse(format!(
            "Statuskode utenfor gyldig område: {}",
            status
        )));
    }

    let meta = if header.len() > 3 {
        header[3..].to_string()
    } else {
        String::new()
    };

    Ok((status, meta))
}

/// Løs en relativ URL mot en Gemini base-URL
pub fn resolve_gemini_url(base: &str, relative: &str) -> Result<String, GeminiError> {
    // Absolutt gemini-URL
    if relative.starts_with("gemini://") {
        return Ok(relative.to_string());
    }

    // Absolutt HTTP/HTTPS-URL — behold som den er
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Ok(relative.to_string());
    }

    let base_url = Url::parse(base)
        .map_err(|e| GeminiError::InvalidUrl(format!("Ugyldig base URL: {}", e)))?;

    let resolved = base_url
        .join(relative)
        .map_err(|e| GeminiError::InvalidUrl(format!("Kunne ikke løse relativ URL: {}", e)))?;

    Ok(resolved.to_string())
}

/// Hent stien til TOFU-lagringsfilen
pub fn get_tofu_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("bare").join("known_hosts.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        let result = GeminiClient::validate_url("gemini://example.com/page");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_with_port() {
        let result = GeminiClient::validate_url("gemini://example.com:1965/page");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_wrong_scheme() {
        let result = GeminiClient::validate_url("https://example.com");
        assert!(matches!(result, Err(GeminiError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_url_no_host() {
        let result = GeminiClient::validate_url("gemini:///path");
        assert!(matches!(result, Err(GeminiError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_url_too_long() {
        let long_url = format!("gemini://example.com/{}", "a".repeat(1024));
        let result = GeminiClient::validate_url(&long_url);
        assert!(matches!(result, Err(GeminiError::InvalidUrl(_))));
    }

    #[test]
    fn test_parse_response_header_success() {
        let (status, meta) = parse_response_header("20 text/gemini\r\n").unwrap();
        assert_eq!(status, 20);
        assert_eq!(meta, "text/gemini");
    }

    #[test]
    fn test_parse_response_header_redirect() {
        let (status, meta) = parse_response_header("31 gemini://example.com/new\r\n").unwrap();
        assert_eq!(status, 31);
        assert_eq!(meta, "gemini://example.com/new");
    }

    #[test]
    fn test_parse_response_header_input() {
        let (status, meta) = parse_response_header("10 Enter a search term\r\n").unwrap();
        assert_eq!(status, 10);
        assert_eq!(meta, "Enter a search term");
    }

    #[test]
    fn test_parse_response_header_error() {
        let (status, meta) = parse_response_header("51 Not found\r\n").unwrap();
        assert_eq!(status, 51);
        assert_eq!(meta, "Not found");
    }

    #[test]
    fn test_parse_response_header_invalid() {
        let result = parse_response_header("X");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_header_empty_meta() {
        let (status, meta) = parse_response_header("20\r\n").unwrap();
        assert_eq!(status, 20);
        assert_eq!(meta, "");
    }

    #[test]
    fn test_parse_response_header_out_of_range() {
        let result = parse_response_header("99 Bad\r\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_tofu_store_new_host() {
        let mut store = TofuStore::default();
        let result = store.verify("example.com:1965", "abc123");
        assert!(result.is_ok());
        assert!(store.hosts.contains_key("example.com:1965"));
    }

    #[test]
    fn test_tofu_store_known_host() {
        let mut store = TofuStore::default();
        store.verify("example.com:1965", "abc123").unwrap();
        let result = store.verify("example.com:1965", "abc123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tofu_store_changed_cert() {
        let mut store = TofuStore::default();
        store.verify("example.com:1965", "abc123").unwrap();
        let result = store.verify("example.com:1965", "different456");
        assert!(matches!(
            result,
            Err(GeminiError::CertificateChanged { .. })
        ));
    }

    #[test]
    fn test_resolve_gemini_url_absolute() {
        let result = resolve_gemini_url("gemini://example.com/dir/page", "gemini://other.com/test");
        assert_eq!(result.unwrap(), "gemini://other.com/test");
    }

    #[test]
    fn test_resolve_gemini_url_relative() {
        let result = resolve_gemini_url("gemini://example.com/dir/page", "other.gmi");
        assert_eq!(result.unwrap(), "gemini://example.com/dir/other.gmi");
    }

    #[test]
    fn test_resolve_gemini_url_parent() {
        let result = resolve_gemini_url("gemini://example.com/dir/page", "../other.gmi");
        assert_eq!(result.unwrap(), "gemini://example.com/other.gmi");
    }

    #[test]
    fn test_resolve_gemini_url_absolute_path() {
        let result = resolve_gemini_url("gemini://example.com/dir/page", "/root.gmi");
        assert_eq!(result.unwrap(), "gemini://example.com/root.gmi");
    }

    #[test]
    fn test_resolve_gemini_url_http() {
        let result = resolve_gemini_url("gemini://example.com/page", "https://web.com");
        assert_eq!(result.unwrap(), "https://web.com");
    }

    #[test]
    fn test_cert_fingerprint_deterministic() {
        let data = vec![1, 2, 3, 4, 5];
        let cert = CertificateDer::from(data.clone());
        let fp1 = cert_fingerprint(&cert);
        let cert2 = CertificateDer::from(data);
        let fp2 = cert_fingerprint(&cert2);
        assert_eq!(fp1, fp2);
        assert!(!fp1.is_empty());
    }

    #[test]
    fn test_tofu_store_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_known_hosts.json");

        let mut store = TofuStore::default();
        store.verify("example.com:1965", "abc123").unwrap();
        store.save(&path).unwrap();

        let loaded = TofuStore::load(&path);
        assert!(loaded.hosts.contains_key("example.com:1965"));
        assert_eq!(loaded.hosts["example.com:1965"].fingerprint, "abc123");
    }
}
