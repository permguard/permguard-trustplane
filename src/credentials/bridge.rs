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

use crate::credentials::CredentialsManager;
use crate::error::{Error, Result};
use crate::bridge::{
    bridge_service_server::{BridgeService, BridgeServiceServer},
    ExchangeRequest, ExchangeResponse,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};
use tracing::{info, warn};

/// Bridge configuration
#[derive(Clone, Debug)]
pub struct BridgeConfig {
    pub id: String,
    pub bridge_type: BridgeType,
    pub enabled: bool,
    pub config: BridgeTypeConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BridgeType {
    Jwt,
    // Spiffe, // Future
    // Vc,     // Future
}

#[derive(Clone, Debug)]
pub enum BridgeTypeConfig {
    Jwt(JwtBridgeConfig),
}

#[derive(Clone, Debug)]
pub struct JwtBridgeConfig {
    pub wellknown_url: String,
    pub issuer: String,
    pub audiences: Vec<String>,
    pub mapping: MappingConfig,
}

#[derive(Clone, Debug, Default)]
pub struct MappingConfig {
    pub subject_claim: String,
    pub organization_claim: String,
    pub custom: HashMap<String, String>,
}

/// Bridge registry
pub struct BridgeRegistry {
    bridges: RwLock<HashMap<String, BridgeConfig>>,
}

impl BridgeRegistry {
    pub fn new() -> Self {
        Self {
            bridges: RwLock::new(HashMap::new()),
        }
    }

    pub fn list(&self) -> Vec<BridgeConfig> {
        self.bridges.read().unwrap().values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<BridgeConfig> {
        self.bridges.read().unwrap().get(id).cloned()
    }

    pub fn add(&self, mut config: BridgeConfig) -> Result<String> {
        if config.id.is_empty() {
            config.id = uuid::Uuid::new_v4().to_string();
        }
        let id = config.id.clone();
        self.bridges.write().unwrap().insert(id.clone(), config);
        info!(bridge_id = %id, "Bridge configuration added");
        Ok(id)
    }

    pub fn update(&self, config: BridgeConfig) -> Result<()> {
        let mut bridges = self.bridges.write().unwrap();
        if !bridges.contains_key(&config.id) {
            return Err(Error::NotFound(config.id));
        }
        bridges.insert(config.id.clone(), config);
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        let mut bridges = self.bridges.write().unwrap();
        if bridges.remove(id).is_none() {
            return Err(Error::NotFound(id.to_string()));
        }
        info!(bridge_id = %id, "Bridge configuration removed");
        Ok(())
    }
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge gRPC service implementation
pub struct BridgeServiceImpl {
    credentials: Arc<CredentialsManager>,
    registry: Arc<BridgeRegistry>,
}

impl BridgeServiceImpl {
    pub fn new(credentials: Arc<CredentialsManager>, registry: Arc<BridgeRegistry>) -> Self {
        Self { credentials, registry }
    }

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
        
        // Get bridge configuration
        let bridge = self.registry.get(&req.bridge_id)
            .ok_or_else(|| Status::not_found(format!("Bridge not found: {}", req.bridge_id)))?;
        
        if !bridge.enabled {
            return Err(Status::failed_precondition("Bridge is disabled"));
        }
        
        // Process based on bridge type
        match &bridge.config {
            BridgeTypeConfig::Jwt(jwt_config) => {
                self.exchange_jwt(&req.credential, jwt_config).await
            }
        }
    }
}

impl BridgeServiceImpl {
    async fn exchange_jwt(
        &self,
        credential: &[u8],
        config: &JwtBridgeConfig,
    ) -> std::result::Result<Response<ExchangeResponse>, Status> {
        // TODO: Implement JWT validation and PCA₀ generation
        // 1. Parse JWT
        // 2. Fetch JWKS from wellknown_url
        // 3. Validate signature
        // 4. Check issuer and audience
        // 5. Map claims to PCA₀
        // 6. Sign with CAT key
        
        warn!("JWT bridge exchange not fully implemented yet");
        
        Ok(Response::new(ExchangeResponse {
            pca: vec![],
            error: "Not implemented".to_string(),
        }))
    }
}