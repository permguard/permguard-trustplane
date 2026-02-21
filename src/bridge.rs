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

//! Bridge gRPC service.
//!
//! The Bridge service exchanges external credentials (JWT, SPIFFE, etc.)
//! for an initial PCA₀ (PIC Causal Authority).

use crate::credentials::CredentialsManager;
use crate::error::{Error, Result};
use crate::proto::bridge::{
    bridge_service_server::{BridgeService, BridgeServiceServer},
    ExchangeRequest, ExchangeResponse,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};
use tracing::{info, warn};

// ============================================================================
// Bridge Configuration Types
// ============================================================================

/// Bridge configuration
#[derive(Clone, Debug)]
pub struct BridgeConfig {
    /// Unique identifier
    pub id: String,
    /// Bridge type
    pub bridge_type: BridgeType,
    /// Whether the bridge is enabled
    pub enabled: bool,
    /// Type-specific configuration
    pub config: BridgeTypeConfig,
}

/// Supported bridge types
#[derive(Clone, Debug, PartialEq)]
pub enum BridgeType {
    /// JWT/OIDC token bridge
    Jwt,
    // Spiffe, // Future: SPIFFE SVID bridge
    // Vc,     // Future: Verifiable Credential bridge
}

/// Type-specific bridge configuration
#[derive(Clone, Debug)]
pub enum BridgeTypeConfig {
    /// JWT bridge configuration
    Jwt(JwtBridgeConfig),
}

/// JWT bridge configuration
#[derive(Clone, Debug)]
pub struct JwtBridgeConfig {
    /// OIDC well-known URL for JWKS discovery
    pub wellknown_url: String,
    /// Expected issuer claim
    pub issuer: String,
    /// Allowed audiences
    pub audiences: Vec<String>,
    /// Claim mapping configuration
    pub mapping: MappingConfig,
}

/// Claim to PCA field mapping configuration
#[derive(Clone, Debug, Default)]
pub struct MappingConfig {
    /// Claim to use for subject (default: "sub")
    pub subject_claim: String,
    /// Claim to use for organization (default: "org")
    pub organization_claim: String,
    /// Custom claim mappings
    pub custom: HashMap<String, String>,
}

// ============================================================================
// Bridge Registry
// ============================================================================

/// Registry for bridge configurations
pub struct BridgeRegistry {
    bridges: RwLock<HashMap<String, BridgeConfig>>,
}

impl BridgeRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            bridges: RwLock::new(HashMap::new()),
        }
    }

    /// List all bridge configurations
    pub fn list(&self) -> Vec<BridgeConfig> {
        self.bridges.read().unwrap().values().cloned().collect()
    }

    /// Get a bridge configuration by ID
    pub fn get(&self, id: &str) -> Option<BridgeConfig> {
        self.bridges.read().unwrap().get(id).cloned()
    }

    /// Add a new bridge configuration
    pub fn add(&self, mut config: BridgeConfig) -> Result<String> {
        if config.id.is_empty() {
            config.id = uuid::Uuid::new_v4().to_string();
        }
        let id = config.id.clone();
        self.bridges.write().unwrap().insert(id.clone(), config);
        info!(bridge_id = %id, "Bridge configuration added");
        Ok(id)
    }

    /// Update an existing bridge configuration
    pub fn update(&self, config: BridgeConfig) -> Result<()> {
        let mut bridges = self.bridges.write().unwrap();
        if !bridges.contains_key(&config.id) {
            return Err(Error::NotFound(config.id));
        }
        info!(bridge_id = %config.id, "Bridge configuration updated");
        bridges.insert(config.id.clone(), config);
        Ok(())
    }

    /// Remove a bridge configuration
    pub fn remove(&self, id: &str) -> Result<()> {
        let mut bridges = self.bridges.write().unwrap();
        if bridges.remove(id).is_none() {
            return Err(Error::NotFound(id.to_string()));
        }
        info!(bridge_id = %id, "Bridge configuration removed");
        Ok(())
    }

    /// Get enabled bridge by ID
    pub fn get_enabled(&self, id: &str) -> Option<BridgeConfig> {
        self.get(id).filter(|b| b.enabled)
    }
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Bridge gRPC Service
// ============================================================================

/// Bridge gRPC service implementation
pub struct BridgeServiceImpl {
    credentials: Arc<CredentialsManager>,
    registry: Arc<BridgeRegistry>,
}

impl BridgeServiceImpl {
    /// Create new bridge service
    pub fn new(credentials: Arc<CredentialsManager>, registry: Arc<BridgeRegistry>) -> Self {
        Self { credentials, registry }
    }

    /// Convert to tonic server
    pub fn into_server(self) -> BridgeServiceServer<Self> {
        BridgeServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl BridgeService for BridgeServiceImpl {
    async fn exchange(
        &self,
        request: Request<ExchangeRequest>,
    ) -> std::result::Result<Response<ExchangeResponse>, Status> {
        let req = request.into_inner();
        
        // Validate request
        if req.bridge_id.is_empty() {
            return Ok(Response::new(ExchangeResponse {
                pca: vec![],
                error: "bridge_id is required".to_string(),
            }));
        }

        if req.credential.is_empty() {
            return Ok(Response::new(ExchangeResponse {
                pca: vec![],
                error: "credential is required".to_string(),
            }));
        }

        // Get bridge configuration
        let bridge = match self.registry.get_enabled(&req.bridge_id) {
            Some(b) => b,
            None => {
                return Ok(Response::new(ExchangeResponse {
                    pca: vec![],
                    error: format!("Bridge not found or disabled: {}", req.bridge_id),
                }));
            }
        };
        
        // Process based on bridge type
        let result = match &bridge.config {
            BridgeTypeConfig::Jwt(jwt_config) => {
                self.exchange_jwt(&req.credential, jwt_config).await
            }
        };

        match result {
            Ok(pca) => Ok(Response::new(ExchangeResponse {
                pca,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ExchangeResponse {
                pca: vec![],
                error: e,
            })),
        }
    }
}

impl BridgeServiceImpl {
    /// Exchange JWT token for PCA₀
    async fn exchange_jwt(
        &self,
        credential: &[u8],
        config: &JwtBridgeConfig,
    ) -> std::result::Result<Vec<u8>, String> {
        // TODO: Implement full JWT validation and PCA₀ generation
        //
        // Steps:
        // 1. Parse JWT from credential bytes
        // 2. Fetch JWKS from config.wellknown_url
        // 3. Validate JWT signature using JWKS
        // 4. Verify issuer matches config.issuer
        // 5. Verify audience is in config.audiences
        // 6. Extract claims using config.mapping
        // 7. Create PCA₀ with extracted claims
        // 8. Sign PCA₀ with CAT key from credentials
        // 9. Return CBOR-encoded PCA₀

        let _credentials = self.credentials.current();
        
        // Parse JWT (just to validate it's UTF-8 for now)
        let _jwt_str = std::str::from_utf8(credential)
            .map_err(|_| "Invalid UTF-8 in credential")?;

        warn!(
            wellknown = %config.wellknown_url,
            issuer = %config.issuer,
            "JWT bridge exchange not fully implemented yet"
        );
        
        Err("JWT bridge not fully implemented".to_string())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_registry_crud() {
        let registry = BridgeRegistry::new();
        
        // Add
        let config = BridgeConfig {
            id: String::new(),
            bridge_type: BridgeType::Jwt,
            enabled: true,
            config: BridgeTypeConfig::Jwt(JwtBridgeConfig {
                wellknown_url: "https://auth.example.com/.well-known/openid-configuration".into(),
                issuer: "https://auth.example.com".into(),
                audiences: vec!["api".into()],
                mapping: MappingConfig::default(),
            }),
        };
        
        let id = registry.add(config).unwrap();
        assert!(!id.is_empty());
        
        // Get
        let fetched = registry.get(&id).unwrap();
        assert_eq!(fetched.bridge_type, BridgeType::Jwt);
        assert!(fetched.enabled);
        
        // List
        let all = registry.list();
        assert_eq!(all.len(), 1);
        
        // Update
        let mut updated = fetched.clone();
        updated.enabled = false;
        registry.update(updated).unwrap();
        
        let fetched2 = registry.get(&id).unwrap();
        assert!(!fetched2.enabled);
        
        // Get enabled (should be None now)
        assert!(registry.get_enabled(&id).is_none());
        
        // Remove
        registry.remove(&id).unwrap();
        assert!(registry.get(&id).is_none());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_bridge_registry_not_found() {
        let registry = BridgeRegistry::new();
        
        assert!(registry.get("nonexistent").is_none());
        assert!(registry.remove("nonexistent").is_err());
        
        let config = BridgeConfig {
            id: "test".into(),
            bridge_type: BridgeType::Jwt,
            enabled: true,
            config: BridgeTypeConfig::Jwt(JwtBridgeConfig {
                wellknown_url: String::new(),
                issuer: String::new(),
                audiences: vec![],
                mapping: MappingConfig::default(),
            }),
        };
        
        // Update non-existent should fail
        assert!(registry.update(config).is_err());
    }
}