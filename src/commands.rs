use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "Lynx.fm CLI - Stream music from your Lynx.fm server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Configure the CLI with Supabase and server URLs
    Config {
        /// Supabase URL
        #[arg(long)]
        supabase_url: Option<String>,
        
        /// Supabase anonymous key
        #[arg(long)]
        supabase_key: Option<String>,
        
        /// Lynx.fm server URL
        #[arg(long)]
        server_url: Option<String>,
    },
    
    /// Sign up for a new account
    Signup,
    
    /// Log in to your account
    Login,
    
    /// Log out from your account
    Logout,
    
    /// Check if the server is healthy
    Health,
    
    /// Play a random track
    Random,
    
    /// Play a specific track
    Play {
        /// Track ID to play
        track_id: String,
    },
    
    /// Prefetch tracks for faster playback
    Prefetch {
        /// Track IDs to prefetch
        track_ids: Vec<String>,
    },
} 