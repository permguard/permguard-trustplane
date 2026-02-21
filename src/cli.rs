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

//! CLI argument definitions.

use clap::Parser;

/// Permguard Trust Plane - PIC-Native Causal Authority Transition Engine
#[derive(Parser, Debug)]
#[command(name = "permguard-trustplane")]
#[command(about = "Permguard Trust Plane\nCopyright © 2026 Nitro Agility S.r.l.\n\nPIC-Native Causal Authority Transition Engine.")]
#[command(version)]
pub struct Cli {
    // === General ===
    /// Enable debug mode (human-readable logs)
    #[arg(long, env = "PERMGUARD_DEBUG", default_value = "false")]
    pub debug: bool,

    /// Log level
    #[arg(long, env = "PERMGUARD_LOG_LEVEL", default_value = "INFO")]
    pub log_level: String,

    /// Directory for application data
    #[arg(long, env = "PERMGUARD_APPDATA", default_value = "./")]
    pub appdata: String,

    // === Identity ===
    /// Trust Plane DID
    #[arg(long, env = "PERMGUARD_DID", default_value = "did:web:localhost")]
    pub did: String,

    /// Organization name
    #[arg(long, env = "PERMGUARD_ORGANIZATION", default_value = "Permguard")]
    pub organization: String,

    /// Credential provider: inmemory, file
    #[arg(long, env = "PERMGUARD_CREDENTIAL_PROVIDER", default_value = "inmemory")]
    pub credential_provider: String,

    /// Path to issuer private key (file provider)
    #[arg(long, env = "PERMGUARD_ISSUER_KEY_PATH")]
    pub issuer_key_path: Option<String>,

    /// Path to CAT private key (file provider)
    #[arg(long, env = "PERMGUARD_CAT_KEY_PATH")]
    pub cat_key_path: Option<String>,

    /// Path to DID document (file provider)
    #[arg(long, env = "PERMGUARD_DID_DOC_PATH")]
    pub did_doc_path: Option<String>,

    /// Path to self-issued credential (file provider)
    #[arg(long, env = "PERMGUARD_CREDENTIAL_PATH")]
    pub credential_path: Option<String>,

    // === Server ===
    /// Bind address
    #[arg(long, env = "PERMGUARD_BIND_ADDRESS", default_value = "0.0.0.0")]
    pub bind_address: String,

    /// HTTP Gateway port (REST API)
    #[arg(long, env = "PERMGUARD_HTTP_PORT", default_value = "9000")]
    pub http_port: u16,

    /// gRPC port
    #[arg(long, env = "PERMGUARD_GRPC_PORT", default_value = "9001")]
    pub grpc_port: u16,

    /// Enable metrics endpoint
    #[arg(long, env = "PERMGUARD_METRICS_ENABLED", default_value = "true")]
    pub metrics_enabled: bool,

    /// Enable Bridge Admin service (disabled by default for security)
    #[arg(long, env = "PERMGUARD_BRIDGE_ADMIN_ENABLED", default_value = "false")]
    pub bridge_admin_enabled: bool,

    // === Shutdown ===
    /// Shutdown grace period in seconds
    #[arg(long, env = "PERMGUARD_SHUTDOWN_GRACE_PERIOD", default_value = "30")]
    pub shutdown_grace_period: u64,
}