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

//! HTTP Gateway handlers.
//!
//! Exposes all services as REST API on the HTTP port.

use crate::bridge::BridgeRegistry;
use crate::credentials::CredentialsManager;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub credentials: Arc<CredentialsManager>,
    pub registry: Arc<BridgeRegistry>,
}

// ============================================================================
// Discovery Handlers
// ============================================================================

/// GET /.well-known/did.json
pub async fn did_document(State(state): State<AppState>) -> Json<serde_json::Value> {
    let creds = state.credentials.current();
    Json(creds.did_document.to_json())
}

/// GET /.well-known/trustplane.json
pub async fn trustplane_metadata(State(state): State<AppState>) -> Json<serde_json::Value> {
    let creds = state.credentials.current();
    Json(serde_json::json!({
        "did": creds.did,
        "organization": creds.organization,
        "issuer_kid": creds.issuer_key.kid(),
        "cat_kid": creds.cat_key.kid(),
        "issuer_public_key": creds.issuer_key.public_jwk(),
        "cat_public_key": creds.cat_key.public_jwk(),
        "credential": creds.credential,
    }))
}

// ============================================================================
// Health Handlers
// ============================================================================

/// GET /health
pub async fn health() -> &'static str {
    "OK"
}

/// GET /ready
pub async fn ready() -> &'static str {
    "OK"
}

/// GET /metrics
pub async fn metrics() -> String {
    // TODO: Implement Prometheus metrics
    "# HELP trustplane_up Trust Plane is up\n# TYPE trustplane_up gauge\ntrustplane_up 1\n"
        .to_string()
}

// ============================================================================
// CAT HTTP Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CatTransitionRequest {
    /// Base64-encoded PCA
    pub pca: String,
}

#[derive(Debug, Serialize)]
pub struct CatTransitionResponse {
    /// Base64-encoded new PCA (empty on error)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub pca: String,
    /// Error message (empty on success)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// POST /v1/cat/transition
pub async fn cat_transition(
    State(state): State<AppState>,
    Json(req): Json<CatTransitionRequest>,
) -> (StatusCode, Json<CatTransitionResponse>) {
    if req.pca.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CatTransitionResponse {
                pca: String::new(),
                error: "pca is required".to_string(),
            }),
        );
    }

    // Decode base64
    let pca_bytes = match base64::engine::general_purpose::STANDARD.decode(&req.pca) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CatTransitionResponse {
                    pca: String::new(),
                    error: format!("Invalid base64: {}", e),
                }),
            );
        }
    };

    // TODO: Implement actual CAT transition
    let _credentials = state.credentials.current();
    let _ = pca_bytes;

    (
        StatusCode::NOT_IMPLEMENTED,
        Json(CatTransitionResponse {
            pca: String::new(),
            error: "Not implemented".to_string(),
        }),
    )
}

// ============================================================================
// Bridge HTTP Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BridgeExchangeRequest {
    /// Bridge configuration ID
    pub bridge_id: String,
    /// Base64-encoded credential
    pub credential: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeExchangeResponse {
    /// Base64-encoded PCA₀ (empty on error)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub pca: String,
    /// Error message (empty on success)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// POST /v1/bridge/exchange
pub async fn bridge_exchange(
    State(state): State<AppState>,
    Json(req): Json<BridgeExchangeRequest>,
) -> (StatusCode, Json<BridgeExchangeResponse>) {
    if req.bridge_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(BridgeExchangeResponse {
                pca: String::new(),
                error: "bridge_id is required".to_string(),
            }),
        );
    }

    if req.credential.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(BridgeExchangeResponse {
                pca: String::new(),
                error: "credential is required".to_string(),
            }),
        );
    }

    // Check bridge exists and is enabled
    match state.registry.get_enabled(&req.bridge_id) {
        Some(_bridge) => {
            // TODO: Implement actual bridge exchange
            (
                StatusCode::NOT_IMPLEMENTED,
                Json(BridgeExchangeResponse {
                    pca: String::new(),
                    error: "Not implemented".to_string(),
                }),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(BridgeExchangeResponse {
                pca: String::new(),
                error: format!("Bridge not found or disabled: {}", req.bridge_id),
            }),
        ),
    }
}

// ============================================================================
// Bridge Admin HTTP Handlers
// ============================================================================

#[derive(Debug, Serialize)]
pub struct BridgeInfo {
    pub id: String,
    pub bridge_type: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ListBridgesResponse {
    pub bridges: Vec<BridgeInfo>,
}

/// GET /v1/bridge-admin/bridges
pub async fn list_bridges(State(state): State<AppState>) -> Json<ListBridgesResponse> {
    let bridges = state
        .registry
        .list()
        .into_iter()
        .map(|b| BridgeInfo {
            id: b.id,
            bridge_type: format!("{:?}", b.bridge_type),
            enabled: b.enabled,
        })
        .collect();

    Json(ListBridgesResponse { bridges })
}

/// GET /v1/bridge-admin/bridges/:id
pub async fn get_bridge(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.registry.get(&id) {
        Some(b) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": b.id,
                "type": format!("{:?}", b.bridge_type),
                "enabled": b.enabled,
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Bridge not found: {}", id) })),
        ),
    }
}

/// DELETE /v1/bridge-admin/bridges/:id
pub async fn remove_bridge(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.registry.remove(&id) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}