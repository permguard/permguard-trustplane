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

//! Bridge Admin gRPC service.

use crate::bridge::{BridgeConfig, BridgeRegistry, BridgeType, BridgeTypeConfig, JwtBridgeConfig, MappingConfig};
use crate::proto::bridge_admin::{
    bridge_admin_service_server::{BridgeAdminService, BridgeAdminServiceServer},
    AddBridgeRequest, AddBridgeResponse,
    GetBridgeRequest, GetBridgeResponse,
    ListBridgesRequest, ListBridgesResponse,
    RemoveBridgeRequest, RemoveBridgeResponse,
    UpdateBridgeRequest, UpdateBridgeResponse,
    BridgeConfig as ProtoBridgeConfig,
    BridgeType as ProtoBridgeType,
    JwtBridgeConfig as ProtoJwtBridgeConfig,
    MappingConfig as ProtoMappingConfig,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// Bridge Admin gRPC service implementation
pub struct BridgeAdminServiceImpl {
    registry: Arc<BridgeRegistry>,
}

impl BridgeAdminServiceImpl {
    pub fn new(registry: Arc<BridgeRegistry>) -> Self {
        Self { registry }
    }

    pub fn into_server(self) -> BridgeAdminServiceServer<Self> {
        BridgeAdminServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl BridgeAdminService for BridgeAdminServiceImpl {
    async fn list_bridges(
        &self,
        _request: Request<ListBridgesRequest>,
    ) -> std::result::Result<Response<ListBridgesResponse>, Status> {
        let bridges = self.registry.list()
            .into_iter()
            .map(to_proto_bridge_config)
            .collect();
        
        Ok(Response::new(ListBridgesResponse { bridges }))
    }

    async fn get_bridge(
        &self,
        request: Request<GetBridgeRequest>,
    ) -> std::result::Result<Response<GetBridgeResponse>, Status> {
        let req = request.into_inner();
        
        match self.registry.get(&req.id) {
            Some(bridge) => Ok(Response::new(GetBridgeResponse {
                bridge: Some(to_proto_bridge_config(bridge)),
                error: String::new(),
            })),
            None => Ok(Response::new(GetBridgeResponse {
                bridge: None,
                error: format!("Bridge not found: {}", req.id),
            })),
        }
    }

    async fn add_bridge(
        &self,
        request: Request<AddBridgeRequest>,
    ) -> std::result::Result<Response<AddBridgeResponse>, Status> {
        let req = request.into_inner();
        
        let bridge = req.bridge
            .ok_or_else(|| Status::invalid_argument("Bridge config required"))?;
        
        let config = from_proto_bridge_config(bridge)?;
        
        match self.registry.add(config) {
            Ok(id) => Ok(Response::new(AddBridgeResponse {
                id,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(AddBridgeResponse {
                id: String::new(),
                error: e.to_string(),
            })),
        }
    }

    async fn update_bridge(
        &self,
        request: Request<UpdateBridgeRequest>,
    ) -> std::result::Result<Response<UpdateBridgeResponse>, Status> {
        let req = request.into_inner();
        
        let bridge = req.bridge
            .ok_or_else(|| Status::invalid_argument("Bridge config required"))?;
        
        let config = from_proto_bridge_config(bridge)?;
        
        match self.registry.update(config) {
            Ok(()) => Ok(Response::new(UpdateBridgeResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(UpdateBridgeResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn remove_bridge(
        &self,
        request: Request<RemoveBridgeRequest>,
    ) -> std::result::Result<Response<RemoveBridgeResponse>, Status> {
        let req = request.into_inner();
        
        match self.registry.remove(&req.id) {
            Ok(()) => Ok(Response::new(RemoveBridgeResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(RemoveBridgeResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }
}

// Conversion helpers

fn to_proto_bridge_config(config: BridgeConfig) -> ProtoBridgeConfig {
    let bridge_type = match config.bridge_type {
        BridgeType::Jwt => ProtoBridgeType::Jwt as i32,
    };

    let config_oneof = match config.config {
        BridgeTypeConfig::Jwt(jwt) => {
            let jwt_proto = ProtoJwtBridgeConfig {
                wellknown_url: jwt.wellknown_url,
                issuer: jwt.issuer,
                audiences: jwt.audiences,
                mapping: Some(ProtoMappingConfig {
                    subject_claim: jwt.mapping.subject_claim,
                    organization_claim: jwt.mapping.organization_claim,
                    custom: jwt.mapping.custom,
                }),
            };
            Some(crate::proto::bridge_admin::bridge_config::Config::Jwt(jwt_proto))
        }
    };

    ProtoBridgeConfig {
        id: config.id,
        r#type: bridge_type,
        enabled: config.enabled,
        config: config_oneof,
    }
}

fn from_proto_bridge_config(proto: ProtoBridgeConfig) -> Result<BridgeConfig, Status> {
    let bridge_type = ProtoBridgeType::try_from(proto.r#type)
        .map_err(|_| Status::invalid_argument("Invalid bridge type"))?;
    
    let config = match bridge_type {
        ProtoBridgeType::Jwt => {
            // Extract JWT config from oneof
            let jwt = match proto.config {
                Some(crate::proto::bridge_admin::bridge_config::Config::Jwt(j)) => j,
                _ => return Err(Status::invalid_argument("JWT config required for JWT bridge")),
            };
            let mapping = jwt.mapping.unwrap_or_default();
            
            BridgeTypeConfig::Jwt(JwtBridgeConfig {
                wellknown_url: jwt.wellknown_url,
                issuer: jwt.issuer,
                audiences: jwt.audiences,
                mapping: MappingConfig {
                    subject_claim: if mapping.subject_claim.is_empty() { 
                        "sub".to_string() 
                    } else { 
                        mapping.subject_claim 
                    },
                    organization_claim: if mapping.organization_claim.is_empty() {
                        "org".to_string()
                    } else {
                        mapping.organization_claim
                    },
                    custom: mapping.custom,
                },
            })
        }
        _ => return Err(Status::invalid_argument("Unsupported bridge type")),
    };
    
    Ok(BridgeConfig {
        id: proto.id,
        bridge_type: BridgeType::Jwt,
        enabled: proto.enabled,
        config,
    })
}