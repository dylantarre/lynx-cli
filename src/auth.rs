use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use dialoguer::{Input, Password};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

use crate::config::Config;

#[derive(Debug, Serialize)]
struct SignUpRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct SignInRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct VerifyOtpRequest {
    email: String,
    token: String,
    type_: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    user: Option<User>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct User {
    id: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

pub struct AuthClient {
    config: Config,
    client: reqwest::Client,
}

impl AuthClient {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(StdDuration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
            
        Self { config, client }
    }
    
    pub async fn signup(&self, email: &str, password: &str) -> Result<()> {
        let url = format!("{}/auth/v1/signup", self.config.supabase_url);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .header("Content-Type", "application/json")
            .json(&SignUpRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .context("Failed to send signup request")?;
            
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await
                .context("Failed to parse error response")?;
                
            anyhow::bail!("Signup failed: {}", error.error_description.unwrap_or(error.error));
        }
        
        println!("Signup successful! Please check your email for a verification code.");
        Ok(())
    }
    
    pub async fn verify_otp(&self, email: &str, token: &str) -> Result<Config> {
        let url = format!("{}/auth/v1/verify", self.config.supabase_url);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .header("Content-Type", "application/json")
            .json(&VerifyOtpRequest {
                email: email.to_string(),
                token: token.to_string(),
                type_: "signup".to_string(),
            })
            .send()
            .await
            .context("Failed to send verification request")?;
            
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await
                .context("Failed to parse error response")?;
                
            anyhow::bail!("Verification failed: {}", error.error_description.unwrap_or(error.error));
        }
        
        let auth_data: AuthResponse = response.json().await
            .context("Failed to parse auth response")?;
            
        let expiry = Utc::now() + Duration::seconds(auth_data.expires_in);
        
        let mut new_config = self.config.clone();
        new_config.auth_token = Some(auth_data.access_token);
        new_config.refresh_token = Some(auth_data.refresh_token);
        new_config.token_expiry = Some(expiry.timestamp());
        
        new_config.save()?;
        
        println!("Email verification successful! You are now logged in.");
        Ok(new_config)
    }
    
    pub async fn login(&self, email: &str, password: &str) -> Result<Config> {
        let url = format!("{}/auth/v1/token?grant_type=password", self.config.supabase_url);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .header("Content-Type", "application/json")
            .json(&SignInRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .context("Failed to send login request")?;
            
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await
                .context("Failed to parse error response")?;
                
            anyhow::bail!("Login failed: {}", error.error_description.unwrap_or(error.error));
        }
        
        let auth_data: AuthResponse = response.json().await
            .context("Failed to parse auth response")?;
            
        let expiry = Utc::now() + Duration::seconds(auth_data.expires_in);
        
        let mut new_config = self.config.clone();
        new_config.auth_token = Some(auth_data.access_token);
        new_config.refresh_token = Some(auth_data.refresh_token);
        new_config.token_expiry = Some(expiry.timestamp());
        
        new_config.save()?;
        
        println!("Login successful!");
        Ok(new_config)
    }
    
    pub async fn refresh_token(&self) -> Result<Config> {
        if self.config.refresh_token.is_none() {
            anyhow::bail!("No refresh token available");
        }
        
        let url = format!("{}/auth/v1/token?grant_type=refresh_token", self.config.supabase_url);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "refresh_token": self.config.refresh_token.as_ref().unwrap()
            }))
            .send()
            .await
            .context("Failed to send refresh token request")?;
            
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await
                .context("Failed to parse error response")?;
                
            anyhow::bail!("Token refresh failed: {}", error.error_description.unwrap_or(error.error));
        }
        
        let auth_data: AuthResponse = response.json().await
            .context("Failed to parse auth response")?;
            
        let expiry = Utc::now() + Duration::seconds(auth_data.expires_in);
        
        let mut new_config = self.config.clone();
        new_config.auth_token = Some(auth_data.access_token);
        new_config.refresh_token = Some(auth_data.refresh_token);
        new_config.token_expiry = Some(expiry.timestamp());
        
        new_config.save()?;
        
        Ok(new_config)
    }
    
    pub async fn logout(&self) -> Result<Config> {
        if self.config.auth_token.is_none() {
            return Ok(self.config.clone());
        }
        
        let url = format!("{}/auth/v1/logout", self.config.supabase_url);
        
        let _response = self.client
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .header("Authorization", format!("Bearer {}", self.config.auth_token.as_ref().unwrap()))
            .send()
            .await
            .context("Failed to send logout request")?;
            
        let mut new_config = self.config.clone();
        new_config.clear_auth()?;
        
        println!("Logout successful!");
        Ok(new_config)
    }
    
    pub async fn interactive_signup() -> Result<Config> {
        let config = Config::load()?;
        let client = Self::new(config.clone());
        
        println!("=== Create a new account ===");
        
        let email: String = Input::new()
            .with_prompt("Email")
            .interact_text()?;
            
        let password: String = Password::new()
            .with_prompt("Password (min 8 characters)")
            .with_confirmation("Confirm password", "Passwords don't match")
            .interact()?;
            
        client.signup(&email, &password).await?;
        
        println!("Please check your email for a verification code.");
        let token: String = Input::new()
            .with_prompt("Enter verification code")
            .interact_text()?;
            
        let new_config = client.verify_otp(&email, &token).await?;
        Ok(new_config)
    }
    
    pub async fn interactive_login() -> Result<Config> {
        let config = Config::load()?;
        let client = Self::new(config.clone());
        
        println!("=== Login to your account ===");
        
        let email: String = Input::new()
            .with_prompt("Email")
            .interact_text()?;
            
        let password: String = Password::new()
            .with_prompt("Password")
            .interact()?;
            
        let new_config = client.login(&email, &password).await?;
        Ok(new_config)
    }
    
    pub async fn ensure_authenticated() -> Result<Config> {
        let mut config = Config::load()?;
        
        if !config.is_authenticated() {
            if config.refresh_token.is_some() {
                // Try to refresh the token
                let client = Self::new(config.clone());
                match client.refresh_token().await {
                    Ok(new_config) => {
                        config = new_config;
                    }
                    Err(_) => {
                        // If refresh fails, clear auth and prompt for login
                        config.clear_auth()?;
                        config = Self::interactive_login().await?;
                    }
                }
            } else {
                // No refresh token, prompt for login
                println!("You need to log in first.");
                config = Self::interactive_login().await?;
            }
        }
        
        Ok(config)
    }
} 