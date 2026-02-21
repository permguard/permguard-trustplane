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

//! Error types.

use std::fmt;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Trust Plane error types
#[derive(Debug)]
pub enum Error {
    /// Configuration error
    Config(String),

    /// Resource not found
    NotFound(String),

    /// Invalid input
    Invalid(String),

    /// Crypto error
    Crypto(String),

    /// IO error
    Io(std::io::Error),

    /// Transport error
    Transport(String),

    /// Internal error
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "configuration error: {}", msg),
            Error::NotFound(id) => write!(f, "not found: {}", id),
            Error::Invalid(msg) => write!(f, "invalid: {}", msg),
            Error::Crypto(msg) => write!(f, "crypto error: {}", msg),
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::Transport(msg) => write!(f, "transport error: {}", msg),
            Error::Internal(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(e: tonic::transport::Error) -> Self {
        Error::Transport(e.to_string())
    }
}

impl From<tonic_reflection::server::Error> for Error {
    fn from(e: tonic_reflection::server::Error) -> Self {
        Error::Internal(format!("gRPC reflection error: {}", e))
    }
}

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        match e {
            Error::NotFound(msg) => tonic::Status::not_found(msg),
            Error::Invalid(msg) => tonic::Status::invalid_argument(msg),
            Error::Config(msg) => tonic::Status::failed_precondition(msg),
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}