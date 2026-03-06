pub mod adapters;
pub mod config;
pub mod db;
pub mod domain;
pub mod errors;
pub mod ports;
pub mod service;
pub mod singbox;
pub mod utils;

pub use db::connect;
pub use db::DbPool;
pub use domain::{generate_config, RoutingConfig};
pub use errors::{ConfigError, DeployError, EnvError, RepoError};
