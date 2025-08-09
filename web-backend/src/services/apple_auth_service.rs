use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    config::Config,
    error::{AppError, AppResult},
};

/// Apple's public key for JWT verification
#[derive(Debug, Clone, Deserialize)]
pub struct ApplePublicKey {
    pub kty: String,
    pub kid: String,
    pub r#use: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

/// Apple's JWKS (JSON Web Key Set) response
#[derive(Debug, Clone, Deserialize)]
pub struct AppleJwksResponse {
    pub keys: Vec<ApplePublicKey>,
}

/// Apple ID Token claims structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppleIdToken {
    /// Issuer - should always be "https://appleid.apple.com"
    pub iss: String,
    /// Audience - your Services ID
    pub aud: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at time
    pub iat: i64,
    /// Subject - unique stable user identifier from Apple
    pub sub: String,
    /// Email address (may not be present on subsequent logins)
    pub email: Option<String>,
    /// Whether email has been verified by Apple
    pub email_verified: Option<String>,
    /// Whether this is a private relay email
    pub is_private_email: Option<String>,
    /// Authentication time
    pub auth_time: Option<i64>,
    /// Nonce (if provided in request)
    pub nonce: Option<String>,
    /// Hash of authorization code (if present)
    pub c_hash: Option<String>,
}

impl AppleIdToken {
    /// Check if email is a private relay email
    pub fn is_private_email(&self) -> bool {
        self.is_private_email
            .as_ref()
            .map(|s| s == "true")
            .unwrap_or(false)
    }

    /// Check if email has been verified
    pub fn is_email_verified(&self) -> bool {
        self.email_verified
            .as_ref()
            .map(|s| s == "true")
            .unwrap_or(false)
    }
}

/// Apple Authentication Service
pub struct AppleAuthService {
    config: Arc<Config>,
    http_client: Client,
    jwks_cache: Option<(AppleJwksResponse, DateTime<Utc>)>,
}

impl AppleAuthService {
    /// Create new Apple Authentication Service
    pub fn new(config: Arc<Config>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            config,
            http_client,
            jwks_cache: None,
        }
    }

    /// Verify Apple Identity Token
    pub async fn verify_identity_token(&mut self, identity_token: &str) -> AppResult<AppleIdToken> {
        // Decode the header to get the key ID
        let header = decode_header(identity_token).map_err(|e| {
            error!("Failed to decode JWT header: {}", e);
            AppError::AuthenticationError("Invalid token format".to_string())
        })?;

        let kid = header.kid.ok_or_else(|| {
            error!("JWT header missing 'kid' field");
            AppError::AuthenticationError("Token missing key identifier".to_string())
        })?;

        // Get Apple's public keys
        let jwks = self.get_apple_jwks().await?;

        // Find the correct public key
        let public_key = jwks.keys.iter().find(|k| k.kid == kid).ok_or_else(|| {
            error!("No matching public key found for kid: {}", kid);
            AppError::AuthenticationError("Invalid key identifier".to_string())
        })?;

        // Verify the algorithm is RS256
        if public_key.alg != "RS256" {
            error!("Unexpected algorithm: {}", public_key.alg);
            return Err(AppError::AuthenticationError(
                "Unsupported algorithm".to_string(),
            ));
        }

        // Create RSA public key from modulus and exponent
        let decoding_key = self.create_decoding_key(public_key)?;

        // Set up validation parameters
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.apple_client_id]);
        validation.set_issuer(&["https://appleid.apple.com"]);
        validation.validate_exp = true;
        validation.validate_nbf = false; // Apple doesn't use nbf
        validation.leeway = 60; // Allow 1 minute clock skew

        // Decode and validate the token
        let token_data = decode::<AppleIdToken>(identity_token, &decoding_key, &validation)
            .map_err(|e| {
                error!("JWT validation failed: {}", e);
                AppError::AuthenticationError(format!("Token validation failed: {}", e))
            })?;

        let claims = token_data.claims;

        // Additional validation checks
        self.validate_apple_claims(&claims)?;

        info!(
            "Successfully verified Apple ID token for user: {}",
            claims.sub
        );
        Ok(claims)
    }

    /// Get Apple's JWKS (JSON Web Key Set)
    async fn get_apple_jwks(&mut self) -> AppResult<AppleJwksResponse> {
        const JWKS_URL: &str = "https://appleid.apple.com/auth/keys";
        const CACHE_DURATION_HOURS: i64 = 24;

        // Check if we have a valid cached JWKS
        if let Some((jwks, cached_at)) = &self.jwks_cache {
            let now = Utc::now();
            if now.signed_duration_since(*cached_at).num_hours() < CACHE_DURATION_HOURS {
                return Ok(jwks.clone());
            }
        }

        // Fetch fresh JWKS from Apple
        let response = self
            .http_client
            .get(JWKS_URL)
            .header("User-Agent", "Learning-App-Backend/1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch Apple JWKS: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            error!(
                "Apple JWKS request failed with status: {}",
                response.status()
            );
            return Err(AppError::InternalServerError);
        }

        let jwks: AppleJwksResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Apple JWKS response: {}", e);
            AppError::InternalServerError
        })?;

        info!(
            "Successfully fetched Apple JWKS with {} keys",
            jwks.keys.len()
        );

        // Cache the JWKS
        self.jwks_cache = Some((jwks.clone(), Utc::now()));
        Ok(jwks)
    }

    /// Create RSA decoding key from Apple's public key components
    fn create_decoding_key(&self, public_key: &ApplePublicKey) -> AppResult<DecodingKey> {
        // Decode base64url-encoded modulus and exponent
        let n_bytes = URL_SAFE_NO_PAD.decode(&public_key.n).map_err(|e| {
            error!("Failed to decode RSA modulus: {}", e);
            AppError::InternalServerError
        })?;

        let e_bytes = URL_SAFE_NO_PAD.decode(&public_key.e).map_err(|e| {
            error!("Failed to decode RSA exponent: {}", e);
            AppError::InternalServerError
        })?;

        // Create RSA public key in DER format
        // This is a simplified approach - in production you might want to use a proper RSA library
        let der_key = self.create_rsa_der_from_components(&n_bytes, &e_bytes)?;

        Ok(DecodingKey::from_rsa_der(&der_key))
    }

    /// Create RSA DER from modulus and exponent components
    /// This is a simplified implementation - for production use a proper RSA library
    fn create_rsa_der_from_components(&self, n: &[u8], e: &[u8]) -> AppResult<Vec<u8>> {
        // This is a basic ASN.1 DER encoding of RSA public key
        // For production, consider using the `rsa` crate for proper handling

        // RSA Public Key ASN.1 structure:
        // SEQUENCE {
        //   SEQUENCE {
        //     OBJECT IDENTIFIER rsaEncryption
        //     NULL
        //   }
        //   BIT STRING {
        //     SEQUENCE {
        //       INTEGER n
        //       INTEGER e
        //     }
        //   }
        // }

        let mut inner_seq = Vec::new();
        // Add INTEGER tag + length + n
        inner_seq.push(0x02); // INTEGER tag
        self.encode_asn1_length(&mut inner_seq, n.len());
        if n[0] & 0x80 != 0 {
            inner_seq.push(0x00); // Add padding if high bit is set
        }
        inner_seq.extend_from_slice(n);

        // Add INTEGER tag + length + e
        inner_seq.push(0x02); // INTEGER tag
        self.encode_asn1_length(&mut inner_seq, e.len());
        if e[0] & 0x80 != 0 {
            inner_seq.push(0x00); // Add padding if high bit is set
        }
        inner_seq.extend_from_slice(e);

        // Wrap in SEQUENCE
        let mut bit_string_content = Vec::new();
        bit_string_content.push(0x30); // SEQUENCE tag
        self.encode_asn1_length(&mut bit_string_content, inner_seq.len());
        bit_string_content.extend(inner_seq);

        // BIT STRING wrapper
        let mut bit_string = Vec::new();
        bit_string.push(0x03); // BIT STRING tag
        self.encode_asn1_length(&mut bit_string, bit_string_content.len() + 1);
        bit_string.push(0x00); // No unused bits
        bit_string.extend(bit_string_content);

        // Algorithm identifier
        let algo_id = vec![
            0x30, 0x0d, // SEQUENCE, length 13
            0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01,
            0x01, // rsaEncryption OID
            0x05, 0x00, // NULL
        ];

        // Final SEQUENCE
        let mut result = Vec::new();
        result.push(0x30); // SEQUENCE tag
        self.encode_asn1_length(&mut result, algo_id.len() + bit_string.len());
        result.extend(algo_id);
        result.extend(bit_string);

        Ok(result)
    }

    /// Encode ASN.1 length field
    fn encode_asn1_length(&self, output: &mut Vec<u8>, length: usize) {
        if length < 0x80 {
            output.push(length as u8);
        } else if length < 0x100 {
            output.push(0x81);
            output.push(length as u8);
        } else if length < 0x10000 {
            output.push(0x82);
            output.push((length >> 8) as u8);
            output.push(length as u8);
        } else {
            // For larger lengths, extend as needed
            output.push(0x83);
            output.push((length >> 16) as u8);
            output.push((length >> 8) as u8);
            output.push(length as u8);
        }
    }

    /// Validate Apple ID token claims
    fn validate_apple_claims(&self, claims: &AppleIdToken) -> AppResult<()> {
        // Verify issuer
        if claims.iss != "https://appleid.apple.com" {
            error!("Invalid issuer: {}", claims.iss);
            return Err(AppError::AuthenticationError(
                "Invalid token issuer".to_string(),
            ));
        }

        // Verify audience
        if claims.aud != self.config.apple_client_id {
            error!(
                "Invalid audience: {} (expected: {})",
                claims.aud, self.config.apple_client_id
            );
            return Err(AppError::AuthenticationError(
                "Invalid token audience".to_string(),
            ));
        }

        // Verify token is not expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if claims.exp < now {
            warn!("Token expired: exp={}, now={}", claims.exp, now);
            return Err(AppError::AuthenticationError("Token expired".to_string()));
        }

        // Verify token is not used before issued
        if claims.iat > now + 60 {
            // Allow 1 minute clock skew
            warn!("Token used before issued: iat={}, now={}", claims.iat, now);
            return Err(AppError::AuthenticationError(
                "Token not yet valid".to_string(),
            ));
        }

        // Verify subject is present and non-empty
        if claims.sub.is_empty() {
            error!("Empty subject in token");
            return Err(AppError::AuthenticationError(
                "Invalid token subject".to_string(),
            ));
        }

        // Check authentication time if present
        if let Some(auth_time) = claims.auth_time {
            if auth_time > now + 60 {
                // Allow 1 minute clock skew
                warn!("Invalid auth_time: {}, now={}", auth_time, now);
                return Err(AppError::AuthenticationError(
                    "Invalid authentication time".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check Apple credential state (for background validation)
    /// Note: This uses Apple's REST API which has rate limits
    pub async fn check_credential_state(&self, apple_user_id: &str) -> AppResult<String> {
        // This is a placeholder for the actual credential state check
        // In practice, you would use Apple's server-to-server API
        // But Apple doesn't provide a direct REST API for this

        // For now, return "unknown" - the real validation happens
        // through refresh token validation or re-authentication
        warn!(
            "Credential state check not fully implemented for user: {}",
            apple_user_id
        );
        Ok("unknown".to_string())
    }

    /// Generate client secret for Apple's server-to-server API calls
    pub fn generate_client_secret(&self) -> AppResult<String> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
        use std::fs;

        // Read the private key file
        let private_key_pem =
            fs::read_to_string(&self.config.apple_private_key_path).map_err(|e| {
                error!("Failed to read Apple private key file: {}", e);
                AppError::InternalServerError
            })?;

        let encoding_key = EncodingKey::from_ec_pem(private_key_pem.as_bytes()).map_err(|e| {
            error!("Failed to parse Apple private key: {}", e);
            AppError::InternalServerError
        })?;

        // Create JWT header
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.config.apple_key_id.clone());

        // Create JWT claims
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let claims = serde_json::json!({
            "iss": self.config.apple_team_id,
            "aud": "https://appleid.apple.com",
            "sub": self.config.apple_client_id,
            "iat": now,
            "exp": now + 3600, // 1 hour expiration
        });

        // Generate the JWT
        encode(&header, &claims, &encoding_key).map_err(|e| {
            error!("Failed to generate client secret: {}", e);
            AppError::InternalServerError
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_token_private_email_parsing() {
        let token = AppleIdToken {
            iss: "https://appleid.apple.com".to_string(),
            aud: "test.client".to_string(),
            exp: 1234567890,
            iat: 1234567890,
            sub: "test.user.123".to_string(),
            email: Some("test@privaterelay.appleid.com".to_string()),
            email_verified: Some("true".to_string()),
            is_private_email: Some("true".to_string()),
            auth_time: Some(1234567890),
            nonce: None,
            c_hash: None,
        };

        assert!(token.is_private_email());
        assert!(token.is_email_verified());
    }

    #[test]
    fn test_apple_token_real_email_parsing() {
        let token = AppleIdToken {
            iss: "https://appleid.apple.com".to_string(),
            aud: "test.client".to_string(),
            exp: 1234567890,
            iat: 1234567890,
            sub: "test.user.123".to_string(),
            email: Some("user@example.com".to_string()),
            email_verified: Some("true".to_string()),
            is_private_email: Some("false".to_string()),
            auth_time: Some(1234567890),
            nonce: None,
            c_hash: None,
        };

        assert!(!token.is_private_email());
        assert!(token.is_email_verified());
    }
}
