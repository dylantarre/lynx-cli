mod auth;
mod commands;
mod config;
mod music;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::auth::AuthClient;
use crate::commands::{Cli, Commands};
use crate::config::Config;
use crate::music::MusicClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Execute the appropriate command
    match cli.command {
        Commands::Config { supabase_url, supabase_key, server_url } => {
            configure(supabase_url, supabase_key, server_url).await?;
        }
        Commands::Signup => {
            AuthClient::interactive_signup().await?;
        }
        Commands::Login => {
            AuthClient::interactive_login().await?;
        }
        Commands::Logout => {
            logout().await?;
        }
        Commands::Health => {
            health_check().await?;
        }
        Commands::Random => {
            play_random().await?;
        }
        Commands::Play { track_id } => {
            play_track(&track_id).await?;
        }
        Commands::Prefetch { track_ids } => {
            prefetch_tracks(track_ids).await?;
        }
    }
    
    Ok(())
}

async fn configure(
    supabase_url: Option<String>,
    supabase_key: Option<String>,
    server_url: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    let mut updated = false;
    
    if let Some(url) = supabase_url {
        config.supabase_url = url;
        updated = true;
    }
    
    if let Some(key) = supabase_key {
        config.supabase_anon_key = key;
        updated = true;
    }
    
    if let Some(url) = server_url {
        config.music_server_url = url;
        updated = true;
    }
    
    if updated {
        config.save()?;
        println!("{}", "Configuration updated successfully.".green());
    } else {
        println!("Current configuration:");
        println!("  Supabase URL: {}", config.supabase_url);
        println!("  Music Server URL: {}", config.music_server_url);
        println!("  Authentication: {}", 
            if config.is_authenticated() { 
                "Authenticated".green() 
            } else { 
                "Not authenticated".yellow() 
            }
        );
    }
    
    Ok(())
}

async fn logout() -> Result<()> {
    let config = Config::load()?;
    let client = AuthClient::new(config);
    client.logout().await?;
    Ok(())
}

async fn health_check() -> Result<()> {
    let config = Config::load()?;
    let client = MusicClient::new(config);
    
    match client.health_check().await {
        Ok(true) => {
            println!("{}", "Server is healthy!".green());
            Ok(())
        }
        Ok(false) => {
            println!("{}", "Server responded but may have issues.".yellow());
            Ok(())
        }
        Err(e) => {
            println!("{} {}", "Server health check failed:".red(), e);
            Err(e)
        }
    }
}

async fn play_random() -> Result<()> {
    // Load config without requiring authentication
    let config = Config::load()?;
    let client = MusicClient::new(config);
    
    let track_id = client.get_random_track().await?;
    client.stream_track(&track_id).await?;
    
    Ok(())
}

async fn play_track(track_id: &str) -> Result<()> {
    // Load config without requiring authentication
    let config = Config::load()?;
    let client = MusicClient::new(config);
    
    client.stream_track(track_id).await?;
    
    Ok(())
}

async fn prefetch_tracks(track_ids: Vec<String>) -> Result<()> {
    let config = AuthClient::ensure_authenticated().await?;
    let client = MusicClient::new(config);
    
    client.prefetch_tracks(track_ids).await?;
    
    Ok(())
}
