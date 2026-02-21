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

//! Credential providers.

use crate::credentials::{DidDocument, KeyPair, TrustPlaneCredentials};
use crate::error::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{info, warn};

/// Trait for credential providers (pluggable for enterprise)
#[async_trait]
pub trait CredentialProvider: Send + Sync {
    /// Load credentials
    fn load(&self) -> Result<TrustPlaneCredentials>;

    /// Watch for credential changes (hot-reload)
    async fn watch(&self, tx: watch::Sender<Arc<TrustPlaneCredentials>>) -> Result<()>;
}

/// In-memory provider: generates ephemeral keys at startup
pub struct InMemoryProvider {
    pub did: String,
    pub organization: String,
}

#[async_trait]
impl CredentialProvider for InMemoryProvider {
    fn load(&self) -> Result<TrustPlaneCredentials> {
        warn!("Using in-memory credential provider - keys are ephemeral!");
        
        let date = Utc::now().format("%Y%m");
        let issuer_kid = format!("{}#issuer-key-{}", self.did, date);
        let cat_kid = format!("{}#cat-key-{}", self.did, date);
        
        let issuer_key = KeyPair::generate(&issuer_kid);
        let cat_key = KeyPair::generate(&cat_kid);
        
        let did_document = DidDocument::new(&self.did, &issuer_key, &cat_key);
        
        let credential = create_self_credential(
            &self.did,
            &self.organization,
            &issuer_key,
        );
        
        info!(
            did = %self.did,
            issuer_kid = %issuer_kid,
            cat_kid = %cat_kid,
            "Generated ephemeral credentials"
        );
        
        Ok(TrustPlaneCredentials {
            did: self.did.clone(),
            organization: self.organization.clone(),
            issuer_key,
            cat_key,
            did_document,
            credential,
        })
    }

    async fn watch(&self, _tx: watch::Sender<Arc<TrustPlaneCredentials>>) -> Result<()> {
        // No refresh for in-memory
        Ok(())
    }
}

/// File provider: loads from disk
pub struct FileProvider {
    pub issuer_key_path: PathBuf,
    pub cat_key_path: PathBuf,
    pub did_doc_path: PathBuf,
    pub credential_path: PathBuf,
}

#[async_trait]
impl CredentialProvider for FileProvider {
    fn load(&self) -> Result<TrustPlaneCredentials> {
        info!(
            issuer_key = %self.issuer_key_path.display(),
            cat_key = %self.cat_key_path.display(),
            "Loading credentials from files"
        );
        
        // Load issuer key
        let issuer_jwk: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&self.issuer_key_path)
                .map_err(|e| Error::Io(e))?
        ).map_err(|e| Error::Invalid(format!("Invalid issuer key JSON: {}", e)))?;
        let issuer_key = KeyPair::from_jwk(&issuer_jwk)?;
        
        // Load CAT key
        let cat_jwk: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&self.cat_key_path)
                .map_err(|e| Error::Io(e))?
        ).map_err(|e| Error::Invalid(format!("Invalid CAT key JSON: {}", e)))?;
        let cat_key = KeyPair::from_jwk(&cat_jwk)?;
        
        // Load DID document
        let did_doc_json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&self.did_doc_path)
                .map_err(|e| Error::Io(e))?
        ).map_err(|e| Error::Invalid(format!("Invalid DID document JSON: {}", e)))?;
        let did_document = DidDocument::from_json(&did_doc_json)?;
        
        // Load credential
        let credential: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&self.credential_path)
                .map_err(|e| Error::Io(e))?
        ).map_err(|e| Error::Invalid(format!("Invalid credential JSON: {}", e)))?;
        
        let did = did_document.id.clone();
        let organization = credential["credentialSubject"]["organization"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        
        info!(
            did = %did,
            issuer_kid = %issuer_key.kid(),
            cat_kid = %cat_key.kid(),
            "Loaded credentials from files"
        );
        
        Ok(TrustPlaneCredentials {
            did,
            organization,
            issuer_key,
            cat_key,
            did_document,
            credential,
        })
    }

    async fn watch(&self, tx: watch::Sender<Arc<TrustPlaneCredentials>>) -> Result<()> {
        // TODO: Implement file watching with notify crate
        // For now, no hot-reload
        Ok(())
    }
}

/// Create self-issued Trust Plane credential
fn create_self_credential(
    did: &str,
    organization: &str,
    issuer_key: &KeyPair,
) -> serde_json::Value {
    let now = Utc::now().to_rfc3339();
    let credential_id = format!("urn:uuid:{}", uuid::Uuid::new_v4());
    
    // Note: In production, this should be properly signed
    // For now, we create the structure without cryptographic proof
    serde_json::json!({
        "@context": [
            "https://www.w3.org/2018/credentials/v1",
            "https://permguard.com/credentials/v1"
        ],
        "id": credential_id,
        "type": ["VerifiableCredential", "TrustPlaneCredential"],
        "issuer": did,
        "issuanceDate": now,
        "credentialSubject": {
            "id": did,
            "type": "TrustPlane",
            "organization": organization
        }
    })
}