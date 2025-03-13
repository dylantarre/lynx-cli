use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub music_server_url: String,
    pub auth_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expiry: Option<i64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            supabase_url: "https://your-project.supabase.co".to_string(),
            supabase_anon_key: "your-anon-key".to_string(),
            music_server_url: "https://server.lg.media".to_string(),
            auth_token: None,
            refresh_token: None,
            token_expiry: None,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let mut dir = home_dir().context("Could not find home directory")?;
        dir.push(".lynx-fm");
        
        if !dir.exists() {
            fs::create_dir_all(&dir).context("Failed to create config directory")?;
        }
        
        Ok(dir)
    }
    
    pub fn config_file() -> Result<PathBuf> {
        let mut path = Self::config_dir()?;
        path.push("config.json");
        Ok(path)
    }
    
    pub fn load() -> Result<Self> {
        let path = Self::config_file()?;
        
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(&path)
            .context("Failed to read config file")?;
            
        let config: Self = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
            
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let path = Self::config_file()?;
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
            
        fs::write(&path, content)
            .context("Failed to write config file")?;
            
        Ok(())
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some() && 
        self.token_expiry.is_some() && 
        self.token_expiry.unwrap() > chrono::Utc::now().timestamp()
    }
    
    pub fn clear_auth(&mut self) -> Result<()> {
        self.auth_token = None;
        self.refresh_token = None;
        self.token_expiry = None;
        self.save()
    }
} 