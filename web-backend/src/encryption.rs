use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::error::{AppError, AppResult};

/// Encrypted field structure that stores ciphertext and nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    /// Base64 encoded ciphertext
    pub ciphertext: String,
    /// Base64 encoded nonce
    pub nonce: String,
    /// Version of encryption used (for future migration)
    pub version: u8,
}

/// PII Encryption Service for handling sensitive data
pub struct PiiEncryption {
    cipher: Aes256Gcm,
    key_rotation_version: u8,
}

impl PiiEncryption {
    /// Create a new PII encryption service from a master key
    pub fn new(master_key: &str) -> AppResult<Self> {
        // Derive encryption key from master key using Argon2
        let key = Self::derive_key(master_key)?;
        let cipher = Aes256Gcm::new(&key);
        
        Ok(Self {
            cipher,
            key_rotation_version: 1,
        })
    }
    
    /// Derive a 256-bit key from the master key using Argon2
    fn derive_key(master_key: &str) -> AppResult<Key<Aes256Gcm>> {
        let salt = b"abcdeez_pii_encryption_v1"; // Static salt for key derivation
        let mut key_bytes = [0u8; 32];
        
        Argon2::default()
            .hash_password_into(master_key.as_bytes(), salt, &mut key_bytes)
            .map_err(|_| AppError::InternalServerError)?;
            
        Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
    }
    
    /// Encrypt a string containing PII
    pub fn encrypt(&self, plaintext: &str) -> AppResult<EncryptedField> {
        // Generate a random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the plaintext
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| AppError::InternalServerError)?;
        
        Ok(EncryptedField {
            ciphertext: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(&nonce),
            version: self.key_rotation_version,
        })
    }
    
    /// Decrypt an encrypted field
    pub fn decrypt(&self, encrypted: &EncryptedField) -> AppResult<String> {
        // Check version compatibility
        if encrypted.version != self.key_rotation_version {
            // In production, handle key rotation here
            return Err(AppError::InternalServerError);
        }
        
        // Decode from base64
        let ciphertext = BASE64.decode(&encrypted.ciphertext)
            .map_err(|_| AppError::InternalServerError)?;
        let nonce_bytes = BASE64.decode(&encrypted.nonce)
            .map_err(|_| AppError::InternalServerError)?;
        
        // Convert nonce bytes to correct type
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| AppError::InternalServerError)?;
        
        String::from_utf8(plaintext)
            .map_err(|_| AppError::InternalServerError)
    }
    
    /// Encrypt multiple fields at once
    pub fn encrypt_fields(&self, fields: &[&str]) -> AppResult<Vec<EncryptedField>> {
        fields.iter()
            .map(|field| self.encrypt(field))
            .collect()
    }
    
    /// Check if a field needs encryption (contains PII patterns)
    pub fn needs_encryption(field_name: &str) -> bool {
        const PII_FIELDS: &[&str] = &[
            "email",
            "phone",
            "ssn",
            "social_security",
            "credit_card",
            "bank_account",
            "passport",
            "driver_license",
            "date_of_birth",
            "dob",
            "address",
            "full_name",
            "first_name",
            "last_name",
            "maiden_name",
            "ip_address",
            "device_id",
            "biometric",
        ];
        
        let field_lower = field_name.to_lowercase();
        PII_FIELDS.iter().any(|&pii| field_lower.contains(pii))
    }
}

/// Service for managing encryption keys and rotation
pub struct KeyManagementService {
    current_key: Arc<PiiEncryption>,
    previous_keys: Vec<Arc<PiiEncryption>>, // For decrypting old data during rotation
}

impl KeyManagementService {
    pub fn new(master_key: &str) -> AppResult<Self> {
        let current_key = Arc::new(PiiEncryption::new(master_key)?);
        
        Ok(Self {
            current_key,
            previous_keys: Vec::new(),
        })
    }
    
    /// Get the current encryption service
    pub fn current(&self) -> Arc<PiiEncryption> {
        Arc::clone(&self.current_key)
    }
    
    /// Rotate to a new encryption key
    pub fn rotate_key(&mut self, new_master_key: &str) -> AppResult<()> {
        // Move current key to previous keys
        self.previous_keys.push(Arc::clone(&self.current_key));
        
        // Create new key with incremented version
        let mut new_encryption = PiiEncryption::new(new_master_key)?;
        new_encryption.key_rotation_version = self.current_key.key_rotation_version + 1;
        
        self.current_key = Arc::new(new_encryption);
        Ok(())
    }
    
    /// Decrypt with automatic key selection based on version
    pub fn decrypt_any_version(&self, encrypted: &EncryptedField) -> AppResult<String> {
        // Try current key first
        if encrypted.version == self.current_key.key_rotation_version {
            return self.current_key.decrypt(encrypted);
        }
        
        // Try previous keys
        for key in &self.previous_keys {
            if encrypted.version == key.key_rotation_version {
                return key.decrypt(encrypted);
            }
        }
        
        Err(AppError::InternalServerError)
    }
}

/// Helper functions for database storage
pub mod db_helpers {
    use super::*;
    use serde_json;
    
    /// Convert encrypted field to JSON for database storage
    pub fn encrypted_to_json(encrypted: &EncryptedField) -> AppResult<serde_json::Value> {
        serde_json::to_value(encrypted)
            .map_err(|_| AppError::InternalServerError)
    }
    
    /// Parse encrypted field from database JSON
    pub fn json_to_encrypted(json: &serde_json::Value) -> AppResult<EncryptedField> {
        serde_json::from_value(json.clone())
            .map_err(|_| AppError::InternalServerError)
    }
    
    /// Encrypt and prepare for database storage
    pub fn encrypt_for_db(encryption: &PiiEncryption, value: &str) -> AppResult<serde_json::Value> {
        let encrypted = encryption.encrypt(value)?;
        encrypted_to_json(&encrypted)
    }
    
    /// Decrypt from database storage
    pub fn decrypt_from_db(encryption: &PiiEncryption, json: &serde_json::Value) -> AppResult<String> {
        let encrypted = json_to_encrypted(json)?;
        encryption.decrypt(&encrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let encryption = PiiEncryption::new("test_master_key_123456789012345678901234567890")
            .expect("Should create encryption service");
        
        let plaintext = "user@example.com";
        let encrypted = encryption.encrypt(plaintext)
            .expect("Should encrypt");
        
        let decrypted = encryption.decrypt(&encrypted)
            .expect("Should decrypt");
        
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_different_nonces() {
        let encryption = PiiEncryption::new("test_master_key_123456789012345678901234567890")
            .expect("Should create encryption service");
        
        let plaintext = "sensitive data";
        let encrypted1 = encryption.encrypt(plaintext).expect("Should encrypt");
        let encrypted2 = encryption.encrypt(plaintext).expect("Should encrypt");
        
        // Same plaintext should produce different ciphertexts due to different nonces
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        
        // But both should decrypt to the same value
        assert_eq!(
            encryption.decrypt(&encrypted1).expect("Should decrypt"),
            encryption.decrypt(&encrypted2).expect("Should decrypt")
        );
    }
    
    #[test]
    fn test_pii_field_detection() {
        assert!(PiiEncryption::needs_encryption("email"));
        assert!(PiiEncryption::needs_encryption("user_email"));
        assert!(PiiEncryption::needs_encryption("credit_card_number"));
        assert!(PiiEncryption::needs_encryption("home_address"));
        assert!(!PiiEncryption::needs_encryption("username"));
        assert!(!PiiEncryption::needs_encryption("id"));
    }
}