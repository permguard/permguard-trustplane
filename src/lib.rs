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

 //! Permguard Trust Plane - PIC-Native Causal Authority Transition Engine.

// PIC types re-export
pub mod pic {
    pub use permguard_pic::*;
}

// Core modules
pub mod cli;
pub mod config;
pub mod error;
pub mod handlers;

// Credentials management
pub mod credentials;

// Services
pub mod bridge;
pub mod bridge_admin;
pub mod cat;

// Server
pub mod server;

mod proto;


// Public API
pub use cli::Cli;
pub use config::Config;
pub use error::{Error, Result};
pub use credentials::{TrustPlaneCredentials, CredentialProvider, CredentialsManager};
pub use server::Server;

/// Returns the crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}