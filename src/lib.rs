pub mod auth;
pub mod commands;
pub mod config;
pub mod music;

// Re-export the modules for easier access in tests
pub use auth::AuthClient;
pub use commands::{Cli, Commands};
pub use config::Config;
pub use music::MusicClient; 