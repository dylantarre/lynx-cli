use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::time::Duration;

use crate::config::Config;

pub struct MusicClient {
    pub config: Config,
    client: reqwest::Client,
}

impl MusicClient {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
            
        Self { config, client }
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.config.music_server_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send health check request")?;
            
        Ok(response.status().is_success())
    }
    
    pub async fn get_random_track(&self) -> Result<String> {
        let url = format!("{}/random", self.config.music_server_url);
        println!("Requesting random track from: {}", url);
        
        // The random endpoint is now public, no authentication required
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to get random track")?;
            
        println!("Response status: {}", response.status());
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            println!("Error response body: {}", error);
            anyhow::bail!("Failed to get random track: {}", error);
        }
        
        // Process the successful response
        self.extract_track_id_from_response(response).await
    }
    
    async fn extract_track_id_from_response(&self, response: reqwest::Response) -> Result<String> {
        // First, try to parse as JSON (new format)
        let text = response.text().await?;
        println!("Response body: {}", text);
        
        // Try to parse the JSON response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            // Check if there's a track_id field
            if let Some(track_id) = json.get("track_id").and_then(|v| v.as_str()) {
                println!("Extracted track ID from JSON: {}", track_id);
                return Ok(track_id.to_string());
            }
            
            // Check if there's an id field
            if let Some(track_id) = json.get("id").and_then(|v| v.as_str()) {
                println!("Extracted track ID from JSON: {}", track_id);
                return Ok(track_id.to_string());
            }
            
            println!("No track ID found in JSON response: {:?}", json);
        } else {
            println!("Response is not valid JSON, trying to extract track ID from text");
            // If it's not JSON, try to extract the track ID from the text
            // This is a fallback in case the server returns just the ID as plain text
            let track_id = text.trim();
            if !track_id.is_empty() {
                println!("Using response text as track ID: {}", track_id);
                return Ok(track_id.to_string());
            }
        }
        
        anyhow::bail!("No track ID found in response")
    }
    
    pub async fn stream_track(&self, track_id: &str) -> Result<()> {
        let url = format!("{}/tracks/{}", self.config.music_server_url, track_id);
        
        println!("Streaming track: {}", track_id);
        
        // Try with JWT token (primary method)
        println!("Trying with JWT token...");
        let mut request = self.client.get(&url);
        
        // Add the JWT token
        if let Some(token) = &self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        // Try to stream the track
        let response = request
            .send()
            .await
            .context("Failed to start streaming track")?;
            
        println!("Response status: {}", response.status());
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            println!("Error response body: {}", error);
            
            // Try with Supabase anon key as apikey header (fallback method)
            println!("Trying with Supabase anon key...");
            let response = self.client
                .get(&url)
                .header("apikey", &self.config.supabase_anon_key)
                .send()
                .await
                .context("Failed to stream track with anon key")?;
                
            println!("Response status with anon key: {}", response.status());
            
            if !response.status().is_success() {
                let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                println!("Error response body with anon key: {}", error);
                anyhow::bail!("Failed to stream track: {}", error);
            }
            
            // Process the successful response
            return self.process_stream_response(response).await;
        }
        
        // Process the successful response
        self.process_stream_response(response).await
    }
    
    async fn process_stream_response(&self, response: reqwest::Response) -> Result<()> {
        // Get content length for progress bar
        let content_length = response
            .content_length()
            .unwrap_or(0);
            
        // Create progress bar
        let pb = ProgressBar::new(content_length);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        // Stream the response body
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error while downloading file")?;
            buffer.extend_from_slice(&chunk);
            pb.inc(chunk.len() as u64);
        }
        
        pb.finish_with_message("Download complete");
        
        // Play the audio
        println!("Playing track...");
        self.play_audio(&buffer)?;
        
        Ok(())
    }
    
    fn play_audio(&self, data: &[u8]) -> Result<()> {
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default()
            .context("Failed to get audio output stream")?;
            
        // Create a sink to play the audio
        let sink = Sink::try_new(&stream_handle)
            .context("Failed to create audio sink")?;
            
        // Load the audio data
        let cursor = Cursor::new(data.to_vec());
        let source = Decoder::new(cursor)
            .context("Failed to decode audio data")?;
            
        // Add the source to the sink
        sink.append(source);
        
        // Play the audio
        sink.play();
        
        // Wait for the audio to finish
        sink.sleep_until_end();
        
        Ok(())
    }
    
    pub async fn prefetch_tracks(&self, track_ids: Vec<String>) -> Result<()> {
        let url = format!("{}/prefetch", self.config.music_server_url);
        
        // Try with JWT token (primary method)
        println!("Trying to prefetch tracks with JWT token...");
        let mut request = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "track_ids": track_ids
            }));
        
        // Add the JWT token
        if let Some(token) = &self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        // Try to prefetch tracks
        let response = request
            .send()
            .await
            .context("Failed to prefetch tracks")?;
            
        println!("Response status: {}", response.status());
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            println!("Error response body: {}", error);
            
            // Try with Supabase anon key as apikey header (fallback method)
            println!("Trying to prefetch tracks with Supabase anon key...");
            let response = self.client
                .post(&url)
                .header("apikey", &self.config.supabase_anon_key)
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "track_ids": track_ids
                }))
                .send()
                .await
                .context("Failed to prefetch tracks with anon key")?;
                
            println!("Response status with anon key: {}", response.status());
            
            if !response.status().is_success() {
                let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                println!("Error response body with anon key: {}", error);
                anyhow::bail!("Failed to prefetch tracks: {}", error);
            }
            
            println!("Tracks prefetched successfully");
            return Ok(());
        }
        
        println!("Tracks prefetched successfully");
        Ok(())
    }
} 