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

//! DID Document management.

use crate::credentials::KeyPair;
use serde::{Deserialize, Serialize};

/// DID Document
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    
    pub id: String,
    
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    
    #[serde(rename = "assertionMethod")]
    pub assertion_method: Vec<String>,
    
    #[serde(rename = "authentication")]
    pub authentication: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    
    #[serde(rename = "type")]
    pub method_type: String,
    
    pub controller: String,
    
    #[serde(rename = "publicKeyJwk")]
    pub public_key_jwk: serde_json::Value,
}

impl DidDocument {
    /// Create DID Document for Trust Plane with issuer and CAT keys
    pub fn new(did: &str, issuer_key: &KeyPair, cat_key: &KeyPair) -> Self {
        let issuer_method = VerificationMethod {
            id: issuer_key.kid().to_string(),
            method_type: "Ed25519VerificationKey2020".to_string(),
            controller: did.to_string(),
            public_key_jwk: issuer_key.public_jwk(),
        };

        let cat_method = VerificationMethod {
            id: cat_key.kid().to_string(),
            method_type: "Ed25519VerificationKey2020".to_string(),
            controller: did.to_string(),
            public_key_jwk: cat_key.public_jwk(),
        };

        Self {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            ],
            id: did.to_string(),
            verification_method: vec![issuer_method, cat_method],
            assertion_method: vec![
                issuer_key.kid().to_string(),
                cat_key.kid().to_string(),
            ],
            authentication: vec![
                issuer_key.kid().to_string(),
                cat_key.kid().to_string(),
            ],
        }
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    /// Load from JSON
    pub fn from_json(json: &serde_json::Value) -> crate::error::Result<Self> {
        serde_json::from_value(json.clone())
            .map_err(|e| crate::error::Error::Invalid(format!("Invalid DID document: {}", e)))
    }
}