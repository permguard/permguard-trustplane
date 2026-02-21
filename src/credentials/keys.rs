/*
 * Copyright Nitro Agility S.r.l.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Key pair management.

use crate::error::{Error, Result};
use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use rand::rngs::OsRng;

/// Ed25519 key pair
#[derive(Clone)]
pub struct KeyPair {
    /// Key ID (e.g., did:web:example.com#key-1)
    pub kid: String,
    
    /// Signing key (private)
    signing_key: SigningKey,
    
    /// Verifying key (public)
    verifying_key: VerifyingKey,
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("kid", &self.kid)
            .field("public_key", &"[REDACTED]")
            .finish()
    }
}

impl KeyPair {
    /// Generate new Ed25519 key pair
    pub fn generate(kid: impl Into<String>) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            kid: kid.into(),
            signing_key,
            verifying_key,
        }
    }

    /// Load from private key bytes
    pub fn from_bytes(kid: impl Into<String>, bytes: &[u8]) -> Result<Self> {
        let signing_key = SigningKey::try_from(bytes)
            .map_err(|e| Error::Crypto(format!("Invalid private key: {}", e)))?;
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            kid: kid.into(),
            signing_key,
            verifying_key,
        })
    }

    /// Get key ID
    pub fn kid(&self) -> &str {
        &self.kid
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Get private key bytes
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Sign message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    /// Export public key as JWK
    pub fn public_jwk(&self) -> serde_json::Value {
        let public_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            self.verifying_key.as_bytes(),
        );
        
        serde_json::json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": public_b64,
            "kid": self.kid
        })
    }

    /// Export private key as JWK (be careful!)
    pub fn private_jwk(&self) -> serde_json::Value {
        let public_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            self.verifying_key.as_bytes(),
        );
        let private_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            self.signing_key.as_bytes(),
        );
        
        serde_json::json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": public_b64,
            "d": private_b64,
            "kid": self.kid
        })
    }

    /// Load from JWK
    pub fn from_jwk(jwk: &serde_json::Value) -> Result<Self> {
        let kid = jwk["kid"].as_str()
            .ok_or_else(|| Error::Crypto("Missing kid in JWK".into()))?;
        
        let d = jwk["d"].as_str()
            .ok_or_else(|| Error::Crypto("Missing private key (d) in JWK".into()))?;
        
        let private_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            d,
        ).map_err(|e| Error::Crypto(format!("Invalid base64 in JWK: {}", e)))?;
        
        Self::from_bytes(kid, &private_bytes)
    }
}