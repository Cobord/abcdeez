#![cfg(feature = "tls")]

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, warn};
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::config::Config;

/// Default configuration constants
const DEFAULT_RENEWAL_THRESHOLD_DAYS: u32 = 30;
const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 5;
const DEFAULT_RETRY_BACKOFF_HOURS: u32 = 6;
const DEFAULT_ALERT_THRESHOLD_DAYS: u32 = 7;
const DEFAULT_FORCE_RENEWAL_DAYS: u32 = 1;
const DEFAULT_CERT_PATH: &str = "certs/cert.pem";
const DEFAULT_KEY_PATH: &str = "certs/key.pem";
const MOCK_CERT_VALIDITY_DAYS: i64 = 90;

/// Helper for consistent certificate error logging and storage
struct CertificateLogger {
    retry_count: u32,
}

impl CertificateLogger {
    fn new(retry_count: u32) -> Self {
        Self { retry_count }
    }

    fn log_error(&self, error_type: &str, message: &str, severity: ErrorSeverity) -> CertificateError {
        let error = CertificateError {
            timestamp: Utc::now(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            retry_count: self.retry_count,
            severity: severity.clone(),
        };

        // Log with appropriate level
        match severity {
            ErrorSeverity::Warning => warn!("Certificate {}: {}", error_type, message),
            ErrorSeverity::Error => error!("Certificate {}: {}", error_type, message),
            ErrorSeverity::Critical => {
                error!("CRITICAL Certificate {}: {}", error_type, message);
                // TODO: Send alerts to monitoring systems
            }
        }

        error
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateStatus {
    pub is_valid: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub days_until_expiry: Option<i64>,
    pub issuer: Option<String>,
    pub subject: Option<String>,
    pub last_renewal_attempt: Option<DateTime<Utc>>,
    pub last_renewal_success: Option<DateTime<Utc>>,
    pub renewal_errors: Vec<CertificateError>,
    pub auto_renewal_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateError {
    pub timestamp: DateTime<Utc>,
    pub error_type: String,
    pub message: String,
    pub retry_count: u32,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalConfiguration {
    pub enabled: bool,
    pub renewal_threshold_days: u32,
    pub max_retry_attempts: u32,
    pub retry_backoff_hours: u32,
    pub alert_threshold_days: u32,
    pub force_renewal_days: u32,
}

impl Default for RenewalConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            renewal_threshold_days: DEFAULT_RENEWAL_THRESHOLD_DAYS,
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
            retry_backoff_hours: DEFAULT_RETRY_BACKOFF_HOURS,
            alert_threshold_days: DEFAULT_ALERT_THRESHOLD_DAYS,
            force_renewal_days: DEFAULT_FORCE_RENEWAL_DAYS,
        }
    }
}

#[derive(Clone)]
pub struct TlsManager {
    config: Arc<Config>,
    renewal_config: RenewalConfiguration,
}

impl TlsManager {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            renewal_config: RenewalConfiguration::default(),
        }
    }

    pub fn with_renewal_config(config: Arc<Config>, renewal_config: RenewalConfiguration) -> Self {
        Self {
            config,
            renewal_config,
        }
    }

    pub async fn create_tls_acceptor(&self) -> Result<Option<TlsAcceptor>> {
        if !self.config.tls_enabled {
            info!("TLS disabled in configuration - running HTTP only");
            return Ok(None);
        }

        // Check if certificates exist and are valid
        match self.get_certificate_status().await {
            Ok(status) => {
                if status.is_valid {
                    if let Some(days_left) = status.days_until_expiry {
                        if days_left < self.renewal_config.alert_threshold_days as i64 {
                            warn!(
                                "Certificate expires in {} days - renewal recommended",
                                days_left
                            );
                        }
                    }

                    // Try to load certificates and create TLS acceptor
                    match self.load_certificates().await {
                        Ok(acceptor) => {
                            info!("TLS acceptor created successfully");
                            Ok(Some(acceptor))
                        }
                        Err(e) => {
                            error!("Failed to create TLS acceptor: {}", e);
                            self.log_certificate_error(
                                "tls_acceptor_creation",
                                &e.to_string(),
                                ErrorSeverity::Error,
                            )
                            .await;
                            Ok(None)
                        }
                    }
                } else {
                    warn!("Certificate is invalid - attempting renewal");
                    match self.renew_certificate().await {
                        Ok(_) => {
                            info!("Certificate renewed successfully, creating TLS acceptor");
                            self.load_certificates().await.map(Some)
                        }
                        Err(e) => {
                            error!("Certificate renewal failed: {}", e);
                            self.log_certificate_error(
                                "certificate_renewal",
                                &e.to_string(),
                                ErrorSeverity::Critical,
                            )
                            .await;
                            Ok(None)
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to get certificate status: {}", e);
                self.log_certificate_error("status_check", &e.to_string(), ErrorSeverity::Error)
                    .await;
                Ok(None)
            }
        }
    }

    pub async fn check_certificate_renewal(&self) -> Result<CertificateStatus> {
        let status = self.get_certificate_status().await?;

        if !status.is_valid {
            warn!("Certificate is invalid - immediate renewal required");
            return self.handle_certificate_renewal(true).await;
        }

        if let Some(days_left) = status.days_until_expiry {
            if days_left <= self.renewal_config.force_renewal_days as i64 {
                error!(
                    "Certificate expires in {} days - forcing renewal",
                    days_left
                );
                return self.handle_certificate_renewal(true).await;
            } else if days_left <= self.renewal_config.renewal_threshold_days as i64 {
                info!(
                    "Certificate expires in {} days - scheduling renewal",
                    days_left
                );
                return self.handle_certificate_renewal(false).await;
            } else if days_left <= self.renewal_config.alert_threshold_days as i64 {
                warn!(
                    "Certificate expires in {} days - renewal alert threshold reached",
                    days_left
                );
            }
        }

        Ok(status)
    }

    async fn handle_certificate_renewal(&self, force: bool) -> Result<CertificateStatus> {
        if !self.renewal_config.enabled && !force {
            warn!("Certificate renewal is disabled and not forced");
            return self.get_certificate_status().await;
        }

        let existing_errors = self.get_recent_renewal_errors().await;

        // Check retry backoff
        if let Some(last_error) = existing_errors.first() {
            let hours_since_last_attempt = (Utc::now() - last_error.timestamp).num_hours();
            let required_backoff =
                (last_error.retry_count as i64) * (self.renewal_config.retry_backoff_hours as i64);

            if hours_since_last_attempt < required_backoff && !force {
                debug!(
                    "Skipping renewal attempt - backoff period active ({}h remaining)",
                    required_backoff - hours_since_last_attempt
                );
                return self.get_certificate_status().await;
            }
        }

        // Check max retry attempts
        let recent_attempts = existing_errors.len() as u32;
        if recent_attempts >= self.renewal_config.max_retry_attempts && !force {
            error!(
                "Maximum renewal attempts ({}) exceeded - manual intervention required",
                self.renewal_config.max_retry_attempts
            );
            self.log_certificate_error(
                "max_retries_exceeded",
                &format!(
                    "Maximum {} renewal attempts exceeded",
                    self.renewal_config.max_retry_attempts
                ),
                ErrorSeverity::Critical,
            )
            .await;
            return self.get_certificate_status().await;
        }

        info!(
            "Attempting certificate renewal (attempt {} of {})",
            recent_attempts + 1,
            self.renewal_config.max_retry_attempts
        );

        match self.renew_certificate().await {
            Ok(_) => {
                info!("Certificate renewal successful");
                self.log_renewal_success().await;
                self.get_certificate_status().await
            }
            Err(e) => {
                error!("Certificate renewal failed: {}", e);
                self.log_certificate_error(
                    "renewal_attempt",
                    &e.to_string(),
                    if recent_attempts + 1 >= self.renewal_config.max_retry_attempts {
                        ErrorSeverity::Critical
                    } else {
                        ErrorSeverity::Error
                    },
                )
                .await;
                self.get_certificate_status().await
            }
        }
    }

    async fn renew_certificate(&self) -> Result<()> {
        // Log renewal attempt
        self.update_last_renewal_attempt().await;

        // Placeholder for actual certificate renewal logic
        // In production, this would integrate with ACME/Let's Encrypt or other CA
        if cfg!(feature = "mock_certificate_renewal") {
            // Mock successful renewal for testing
            tokio::time::sleep(Duration::from_millis(100)).await;
            info!("Mock certificate renewal completed successfully");
            return Ok(());
        }

        // For now, return an error to simulate renewal challenges
        anyhow::bail!("Certificate renewal not implemented - would integrate with ACME/Let's Encrypt in production")
    }

    async fn load_certificates(&self) -> Result<TlsAcceptor> {
        // Check if we're in development mode and should use self-signed certificates
        if self.config.environment == crate::config::Environment::Development {
            warn!("Using self-signed certificates for development - DO NOT USE IN PRODUCTION");
            return self.create_self_signed_certificate().await;
        }
        
        // In production, bail out and ask for proper certs (keep compile surface minimal)
        anyhow::bail!(
            "TLS in non-development environments requires valid certificate files. Set TLS_CERT_PATH and TLS_KEY_PATH, or disable TLS."
        )
    }

    async fn create_self_signed_certificate(&self) -> Result<TlsAcceptor> {
        // Generate a minimal self-signed cert using rcgen, then build rustls config
        let domains = vec![self.config.server_name.clone(), "localhost".to_string()];
        let certified = rcgen::generate_simple_self_signed(domains)?;
        // rcgen 0.14 returns a CertifiedKey with cert and signing_key
        let cert_der_vec: Vec<u8> = certified.cert.der().to_vec();
        let key_der_vec: Vec<u8> = certified.signing_key.serialize_der();

        let certs: Vec<CertificateDer<'static>> = vec![CertificateDer::from(cert_der_vec)];
        // rcgen generates PKCS#8 format keys
        let key = PrivateKeyDer::from(rustls::pki_types::PrivatePkcs8KeyDer::from(key_der_vec));

        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| anyhow::anyhow!("Failed to create TLS config: {}", e))?;

        Ok(TlsAcceptor::from(Arc::new(tls_config)))
    }

    async fn get_certificate_status(&self) -> Result<CertificateStatus> {
        // Get certificate information from files or certificate store
        let cert_path = self.get_certificate_path();
        let key_path = self.get_private_key_path();

        if !Path::new(&cert_path).exists() || !Path::new(&key_path).exists() {
            return Ok(CertificateStatus {
                is_valid: false,
                expires_at: None,
                days_until_expiry: None,
                issuer: None,
                subject: None,
                last_renewal_attempt: self.get_last_renewal_attempt().await,
                last_renewal_success: self.get_last_renewal_success().await,
                renewal_errors: self.get_recent_renewal_errors().await,
                auto_renewal_enabled: self.renewal_config.enabled,
            });
        }

        // Parse certificate to get expiration info
        // This is a simplified implementation - production would use proper certificate parsing
        let expires_at = self.parse_certificate_expiry(&cert_path).await?;
        let days_until_expiry = (expires_at - Utc::now()).num_days();

        Ok(CertificateStatus {
            is_valid: days_until_expiry > 0,
            expires_at: Some(expires_at),
            days_until_expiry: Some(days_until_expiry),
            issuer: Some("Unknown".to_string()), // Would parse from actual certificate
            subject: Some("localhost".to_string()), // Would parse from actual certificate
            last_renewal_attempt: self.get_last_renewal_attempt().await,
            last_renewal_success: self.get_last_renewal_success().await,
            renewal_errors: self.get_recent_renewal_errors().await,
            auto_renewal_enabled: self.renewal_config.enabled,
        })
    }

    fn get_certificate_path(&self) -> String {
        self.config
            .tls_cert_path
            .clone()
            .unwrap_or_else(|| DEFAULT_CERT_PATH.to_string())
    }

    fn get_private_key_path(&self) -> String {
        self.config
            .tls_key_path
            .clone()
            .unwrap_or_else(|| DEFAULT_KEY_PATH.to_string())
    }

    async fn parse_certificate_expiry(&self, _cert_path: &str) -> Result<DateTime<Utc>> {
        // TODO: In production, parse actual certificate file using rustls or openssl
        // Return a mock expiry date for testing
        Ok(Utc::now() + chrono::Duration::days(MOCK_CERT_VALIDITY_DAYS))
    }

    async fn log_certificate_error(
        &self,
        error_type: &str,
        message: &str,
        severity: ErrorSeverity,
    ) {
        let logger = CertificateLogger::new(self.get_current_retry_count().await);
        let error = logger.log_error(error_type, message, severity);
        self.store_certificate_error(error).await;
    }

    async fn store_certificate_error(&self, _error: CertificateError) {
        // TODO: Store in database or file system for tracking
    }

    async fn get_recent_renewal_errors(&self) -> Vec<CertificateError> {
        // TODO: Retrieve from storage
        Vec::new()
    }

    async fn get_current_retry_count(&self) -> u32 {
        self.get_recent_renewal_errors().await.len() as u32
    }

    async fn update_renewal_metadata(&self, success: bool) {
        // TODO: Update renewal attempt/success timestamp in persistent storage
        if success {
            debug!("Updated renewal success timestamp");
        } else {
            debug!("Updated renewal attempt timestamp");
        }
    }

    async fn update_last_renewal_attempt(&self) {
        self.update_renewal_metadata(false).await;
    }

    async fn log_renewal_success(&self) {
        self.update_renewal_metadata(true).await;
    }

    async fn get_renewal_metadata(&self, success: bool) -> Option<DateTime<Utc>> {
        // TODO: Retrieve from persistent storage
        // For now return None to indicate no previous attempts
        let _ = success; // Suppress unused parameter warning
        None
    }

    async fn get_last_renewal_attempt(&self) -> Option<DateTime<Utc>> {
        self.get_renewal_metadata(false).await
    }

    async fn get_last_renewal_success(&self) -> Option<DateTime<Utc>> {
        self.get_renewal_metadata(true).await
    }

    /// Get comprehensive certificate health information for monitoring
    pub async fn get_certificate_health(&self) -> CertificateStatus {
        self.get_certificate_status().await.unwrap_or_else(|e| {
            error!("Failed to get certificate status: {}", e);
            let logger = CertificateLogger::new(0);
            let error = logger.log_error("status_check_failure", &e.to_string(), ErrorSeverity::Error);
            
            CertificateStatus {
                is_valid: false,
                expires_at: None,
                days_until_expiry: None,
                issuer: None,
                subject: None,
                last_renewal_attempt: None,
                last_renewal_success: None,
                renewal_errors: vec![error],
                auto_renewal_enabled: self.renewal_config.enabled,
            }
        })
    }

    /// Force certificate renewal (bypass retry limits and backoff)
    pub async fn force_renewal(&self) -> Result<CertificateStatus> {
        warn!("Forcing certificate renewal - bypassing safety checks");
        self.handle_certificate_renewal(true).await
    }
}

/// Handle ACME challenge for Let's Encrypt (placeholder)
pub async fn handle_acme_challenge(
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<String, crate::error::AppError> {
    Err(crate::error::AppError::NotFound(format!(
        "Challenge not found: {}",
        token
    )))
}

/// Serve the application  
pub async fn serve_with_tls(
    app: axum::Router,
    http_addr: SocketAddr,
    _https_addr: SocketAddr,
    _tls_acceptor: Option<TlsAcceptor>,
) -> Result<()> {
    // For now, just run HTTP server
    info!("Starting HTTP server on {}", http_addr);
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
