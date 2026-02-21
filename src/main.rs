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

use clap::Parser;
use permguard_trustplane::{Cli, Config, Server, version};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const ART: &str = include_str!("assets/art.txt");

fn main() {
    // Parse CLI args (also reads env vars via clap)
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug { "DEBUG" } else { &cli.log_level };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    if cli.debug {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .pretty()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    }

    // Print banner
    println!("{}", ART);
    println!();
    println!("The official Permguard TrustPlane v{}", version());
    println!("Copyright © 2026 Nitro Agility S.r.l.");
    println!();

    // Convert CLI to Config
    let config = match Config::try_from(cli) {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Invalid configuration");
            std::process::exit(1);
        }
    };

    info!(version = version(), debug = config.debug, did = %config.did, "Starting Permguard Trust Plane");

    // Build and run server
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
        .block_on(async {
            let server = Server::new(config).await?;
            server.run().await
        });

    if let Err(e) = result {
        error!(error = %e, "Server error");
        std::process::exit(1);
    }
}
