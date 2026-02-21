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

//! Configuration management.

use crate::cli::Cli;
use crate::error::{Error, Result};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub debug: bool,
    pub log_level: String,
    pub appdata: PathBuf,

    // Identity
    pub did: String,
    pub organization: String,

    // Server
    pub http_addr: SocketAddr,
    pub grpc_addr: SocketAddr,
    pub metrics_enabled: bool,
    pub bridge_admin_enabled: bool,
    pub shutdown_grace_period: Duration,
}

impl TryFrom<Cli> for Config {
    type Error = Error;

    fn try_from(cli: Cli) -> Result<Self> {
        let http_addr: SocketAddr = format!("{}:{}", cli.bind_address, cli.http_port)
            .parse()
            .map_err(|e| Error::Config(format!("Invalid HTTP address: {}", e)))?;

        let grpc_addr: SocketAddr = format!("{}:{}", cli.bind_address, cli.grpc_port)
            .parse()
            .map_err(|e| Error::Config(format!("Invalid gRPC address: {}", e)))?;

        Ok(Self {
            debug: cli.debug,
            log_level: cli.log_level,
            appdata: PathBuf::from(cli.appdata),
            did: cli.did,
            organization: cli.organization,
            http_addr,
            grpc_addr,
            metrics_enabled: cli.metrics_enabled,
            bridge_admin_enabled: cli.bridge_admin_enabled,
            shutdown_grace_period: Duration::from_secs(cli.shutdown_grace_period),
        })
    }
}