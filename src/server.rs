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

//! Server orchestration.

use crate::bridge::{BridgeRegistry, BridgeServiceImpl};
use crate::bridge_admin::BridgeAdminServiceImpl;
use crate::cat::CatServiceImpl;
use crate::config::Config;
use crate::credentials::{CredentialsManager, InMemoryProvider};
use crate::error::Result;
use crate::handlers::{self, AppState};
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::signal;
use tonic::transport::Server as TonicServer;
use tracing::info;

/// File descriptor for gRPC reflection
const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("proto/descriptor.bin");

/// Trust Plane server
pub struct Server {
    config: Config,
    credentials: Arc<CredentialsManager>,
    bridge_registry: Arc<BridgeRegistry>,
}

impl Server {
    /// Create new server with default in-memory provider
    pub async fn new(config: Config) -> Result<Self> {
        let provider = InMemoryProvider {
            did: config.did.clone(),
            organization: config.organization.clone(),
        };

        let credentials = Arc::new(CredentialsManager::from_provider(&provider)?);
        let bridge_registry = Arc::new(BridgeRegistry::new());

        Ok(Self {
            config,
            credentials,
            bridge_registry,
        })
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        let http_addr = self.config.http_addr;
        let grpc_addr = self.config.grpc_addr;

        // Shared state for HTTP handlers
        let state = AppState {
            credentials: self.credentials.clone(),
            registry: self.bridge_registry.clone(),
        };

        // ====================================================================
        // HTTP Gateway
        // ====================================================================
        let mut http_router = Router::new()
            // Discovery
            .route("/.well-known/did.json", get(handlers::did_document))
            .route(
                "/.well-known/trustplane.json",
                get(handlers::trustplane_metadata),
            )
            // Health
            .route("/health", get(handlers::health))
            .route("/ready", get(handlers::ready))
            // CAT
            .route("/v1/cat/transition", post(handlers::cat_transition))
            // Bridge
            .route("/v1/bridge/exchange", post(handlers::bridge_exchange));

        // Metrics (optional)
        if self.config.metrics_enabled {
            http_router = http_router.route("/metrics", get(handlers::metrics));
        }

        // Bridge Admin (optional)
        if self.config.bridge_admin_enabled {
            http_router = http_router
                .route("/v1/bridge-admin/bridges", get(handlers::list_bridges))
                .route("/v1/bridge-admin/bridges/:id", get(handlers::get_bridge))
                .route(
                    "/v1/bridge-admin/bridges/:id",
                    delete(handlers::remove_bridge),
                );
        }

        let http_router = http_router.with_state(state);

        // ====================================================================
        // gRPC Server with Reflection
        // ====================================================================
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()?;

        let mut grpc_builder = TonicServer::builder()
            .add_service(reflection_service)
            .add_service(CatServiceImpl::new(self.credentials.clone()).into_server())
            .add_service(
                BridgeServiceImpl::new(self.credentials.clone(), self.bridge_registry.clone())
                    .into_server(),
            );

        // Bridge Admin gRPC (optional)
        if self.config.bridge_admin_enabled {
            grpc_builder = grpc_builder
                .add_service(BridgeAdminServiceImpl::new(self.bridge_registry.clone()).into_server());
        }

        // ====================================================================
        // Logging
        // ====================================================================
        info!("[TRUST-PLANE]: Starting servers");
        info!("");
        info!("  HTTP Gateway: http://{}", http_addr);
        info!("    GET  /.well-known/did.json");
        info!("    GET  /.well-known/trustplane.json");
        info!("    GET  /health");
        info!("    GET  /ready");
        if self.config.metrics_enabled {
            info!("    GET  /metrics");
        }
        info!("    POST /v1/cat/transition");
        info!("    POST /v1/bridge/exchange");
        if self.config.bridge_admin_enabled {
            info!("    GET  /v1/bridge-admin/bridges");
            info!("    GET  /v1/bridge-admin/bridges/:id");
            info!("    DELETE /v1/bridge-admin/bridges/:id");
        }
        info!("");
        info!("  gRPC Server: {}", grpc_addr);
        info!("    CatService.Transition");
        info!("    BridgeService.Exchange");
        if self.config.bridge_admin_enabled {
            info!("    BridgeAdminService.*");
        } else {
            info!("    BridgeAdmin: disabled");
        }
        info!("");

        // ====================================================================
        // Start servers
        // ====================================================================
        let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
        let http_server = axum::serve(http_listener, http_router);

        let grpc_server = grpc_builder.serve(grpc_addr);

        // Run both servers concurrently
        tokio::select! {
            res = http_server => {
                if let Err(e) = res {
                    tracing::error!(error = %e, "HTTP server error");
                }
            }
            res = grpc_server => {
                if let Err(e) = res {
                    tracing::error!(error = %e, "gRPC server error");
                }
            }
            _ = shutdown_signal() => {
                info!("Received shutdown signal");
            }
        }

        info!("Server shutdown complete");
        Ok(())
    }
}

/// Wait for shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received SIGINT"),
        _ = terminate => info!("Received SIGTERM"),
    }

    info!("Shutting down gracefully...");
}