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

 use std::path::Path;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = "src/proto";

    if !Path::new(out_dir).exists() {
        std::fs::create_dir_all(out_dir)?;
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path(format!("{}/descriptor.bin", out_dir))
        .out_dir(out_dir)
        .compile_protos(
            &[
                "proto/cat.proto",
                "proto/bridge.proto",
                "proto/bridge_admin.proto",
            ],
            &["proto/"],
        )?;

    // Aggiungi questo: genera mod.rs
    fs::write(
        format!("{}/mod.rs", out_dir),
        r#"//! Generated protobuf code.

#[path = "permguard.trustplane.cat.v1.rs"]
pub mod cat;

#[path = "permguard.trustplane.bridge.v1.rs"]
pub mod bridge;

#[path = "permguard.trustplane.bridge_admin.v1.rs"]
pub mod bridge_admin;
"#,
    )?;

    Ok(())
}