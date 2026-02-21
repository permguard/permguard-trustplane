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

//! CAT (Causal Authority Transition) gRPC service.

use crate::credentials::CredentialsManager;
use crate::proto::cat::{
    cat_service_server::{CatService, CatServiceServer},
    TransitionRequest, TransitionResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{warn};

/// CAT gRPC service implementation
pub struct CatServiceImpl {
    credentials: Arc<CredentialsManager>,
}

impl CatServiceImpl {
    pub fn new(credentials: Arc<CredentialsManager>) -> Self {
        Self { credentials }
    }

    pub fn into_server(self) -> CatServiceServer<Self> {
        CatServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl CatService for CatServiceImpl {
    async fn transition(
        &self,
        request: Request<TransitionRequest>,
    ) -> std::result::Result<Response<TransitionResponse>, Status> {
        let req = request.into_inner();
        
        if req.pca.is_empty() {
            return Ok(Response::new(TransitionResponse {
                pca: vec![],
                error: "PCA is required".to_string(),
            }));
        }
        
        // TODO: Implement actual PCA transition logic using pic-protocol
        // 1. Decode incoming PCA (CBOR)
        // 2. Validate PCA signature and chain
        // 3. Create new PCA with incremented sequence
        // 4. Sign with CAT key
        // 5. Encode as CBOR
        
        let _credentials = self.credentials.current();
        
        warn!("CAT transition not fully implemented yet");
        
        // Placeholder response
        Ok(Response::new(TransitionResponse {
            pca: vec![],
            error: "Not implemented".to_string(),
        }))
    }
}