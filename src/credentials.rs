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
 *//*
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


//! Trust Plane credentials management.

mod provider;
mod did;
mod keys;

pub use provider::{CredentialProvider, InMemoryProvider, FileProvider};
pub use did::DidDocument;
pub use keys::KeyPair;

use crate::error::Result;
use std::sync::Arc;
use tokio::sync::watch;

/// Trust Plane credentials: DID, keys, and self-issued credential
#[derive(Clone, Debug)]
pub struct TrustPlaneCredentials {
    /// DID (e.g., did:web:trustplane.example.com)
    pub did: String,
    
    /// Organization name
    pub organization: String,
    
    /// Issuer key for signing VCs
    pub issuer_key: KeyPair,
    
    /// CAT key for signing PCAs
    pub cat_key: KeyPair,
    
    /// DID Document
    pub did_document: DidDocument,
    
    /// Self-issued credential
    pub credential: serde_json::Value,
}

/// Manages credentials lifecycle with hot-reload support
pub struct CredentialsManager {
    current: watch::Sender<Arc<TrustPlaneCredentials>>,
    receiver: watch::Receiver<Arc<TrustPlaneCredentials>>,
}

impl CredentialsManager {
    /// Create new manager with initial credentials
    pub fn new(credentials: TrustPlaneCredentials) -> Self {
        let (tx, rx) = watch::channel(Arc::new(credentials));
        Self {
            current: tx,
            receiver: rx,
        }
    }

    /// Create from provider
    pub fn from_provider(provider: &dyn CredentialProvider) -> Result<Self> {
        let credentials = provider.load()?;
        Ok(Self::new(credentials))
    }

    /// Get current credentials
    pub fn current(&self) -> Arc<TrustPlaneCredentials> {
        self.receiver.borrow().clone()
    }

    /// Subscribe to credential updates
    pub fn subscribe(&self) -> watch::Receiver<Arc<TrustPlaneCredentials>> {
        self.receiver.clone()
    }

    /// Update credentials
    pub fn update(&self, credentials: TrustPlaneCredentials) {
        let _ = self.current.send(Arc::new(credentials));
    }

    /// Start watching for credential changes (background task)
    pub async fn start_watch(&self, provider: Box<dyn CredentialProvider>) -> Result<()> {
        provider.watch(self.current.clone()).await
    }
}