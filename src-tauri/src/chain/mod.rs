//! Chain connectivity — RPC clients, balance checking, gas fetching.
//! Provides a registry of supported EVM chains and multi-provider endpoints.

pub mod provider;
pub mod registry;

pub use provider::{ChainClient, ChainClientError};
pub use registry::{ChainConfig, ChainRegistry, SUPPORTED_CHAINS};
