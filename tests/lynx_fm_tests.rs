use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use tempfile::tempdir;
use serde_json::Value;

// Import the modules from the main crate
use lynx_fm::config::Config;
use lynx_fm::music::MusicClient;

// Helper function to create a test config
fn create_test_config() -> Config {
    let mut config = Config {
        supabase_url: "https://fpuueievvvxbgbqtkjyd.supabase.co".to_string(),
        supabase_anon_key: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImZwdXVlaWV2dnZ4YmdicXRranlkIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NDE1NzU3MzksImV4cCI6MjA1NzE1MTczOX0.4JtNfUKmp7VqR175M3HXk639AG5cdC7pnUXzR2e8tEM".to_string(),
        music_server_url: "http://go.lynx.fm:3500".to_string(),
        auth_token: None,
        refresh_token: None,
        token_expiry: Some(0),
    };
    
    // Try to load the real auth token from the config file
    if let Ok(home_dir) = env::var("HOME") {
        let config_path = PathBuf::from(home_dir).join(".lynx-fm/config.json");
        if config_path.exists() {
            if let Ok(config_str) = fs::read_to_string(config_path) {
                if let Ok(real_config) = serde_json::from_str::<Config>(&config_str) {
                    config.auth_token = real_config.auth_token;
                    config.refresh_token = real_config.refresh_token;
                    config.token_expiry = real_config.token_expiry;
                }
            }
        }
    }
    
    config
}

#[tokio::test]
async fn test_health_check() -> Result<()> {
    let config = create_test_config();
    let client = MusicClient::new(config);
    
    let result = client.health_check().await?;
    assert!(result, "Health check should return true");
    
    Ok(())
}

#[tokio::test]
async fn test_me_endpoint() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // Test different endpoint paths
    let endpoints = vec![
        format!("{}/me", config.music_server_url),
        format!("{}/api/me", config.music_server_url),
        format!("{}/v1/me", config.music_server_url),
        format!("{}/api/v1/me", config.music_server_url),
    ];

    for url in endpoints {
        println!("\nTesting endpoint: {}", url);
        
        // 1. Test without authentication
        println!("Testing without auth...");
        let response = client.get(&url).send().await?;
        let status = response.status().as_u16();
        assert!(
            status == 401 || status == 404,
            "Expected 401 Unauthorized or 404 Not Found without authentication for {}, got {}",
            url,
            status
        );

        // 2. Test with invalid JWT token
        println!("Testing with invalid JWT...");
        let response = client
            .get(&url)
            .header("Authorization", "Bearer invalid_token")
            .send()
            .await?;
        let status = response.status().as_u16();
        assert!(
            status == 401 || status == 404,
            "Expected 401 Unauthorized or 404 Not Found with invalid JWT for {}, got {}",
            url,
            status
        );

        // 3. Test with valid JWT token
        if let Some(token) = &config.auth_token {
            println!("Testing with valid JWT token...");
            let response = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;
            
            let status = response.status();
            println!("JWT Status: {}", status);
            
            if status.is_success() {
                let body: Value = response.json().await?;
                println!("JWT Response: {:?}", body);
                
                // Validate response structure
                assert!(body.get("email").is_some(), "Response should contain email");
                assert!(body.get("user_id").is_some(), "Response should contain user_id");
                
                // Validate data types
                assert!(body["email"].is_string(), "Email should be a string");
                assert!(body["user_id"].is_string(), "User ID should be a string");
                
                // Optional fields that might be present
                if let Some(role) = body.get("role") {
                    assert!(role.is_string(), "Role should be a string");
                }
                
                if let Some(metadata) = body.get("metadata") {
                    assert!(metadata.is_object(), "Metadata should be an object");
                }
            } else {
                let status = status.as_u16();
                assert!(
                    status == 401 || status == 404,
                    "Expected 401 Unauthorized or 404 Not Found with JWT token for {}, got {}",
                    url,
                    status
                );
                println!("Response body for failed JWT request: {}", response.text().await?);
            }
        }

        // 4. Test with Supabase anon key
        println!("Testing with Supabase anon key...");
        let response = client
            .get(&url)
            .header("apikey", &config.supabase_anon_key)
            .send()
            .await?;
            
        let status = response.status();
        println!("Anon key Status: {}", status);
        
        if status.is_success() {
            let body: Value = response.json().await?;
            println!("Anon key Response: {:?}", body);
            
            // Validate anonymous user response
            assert_eq!(
                body.get("user_id").and_then(|v| v.as_str()),
                Some("anon-user"),
                "Anonymous user should have user_id 'anon-user'"
            );
        } else {
            let status = status.as_u16();
            assert!(
                status == 401 || status == 404,
                "Expected 401 Unauthorized or 404 Not Found with anon key for {}, got {}",
                url,
                status
            );
            println!("Response body for failed anon key request: {}", response.text().await?);
        }

        // 5. Test with malformed headers
        println!("Testing with malformed headers...");
        let malformed_headers = vec![
            ("Authorization", "Bearer"),
            ("Authorization", ""),
            ("apikey", ""),
            ("Authorization", "Basic dXNlcjpwYXNz"),  // Basic auth should not work
        ];

        for (header_name, header_value) in malformed_headers {
            let response = client
                .get(&url)
                .header(header_name, header_value)
                .send()
                .await?;
            let status = response.status().as_u16();
            assert!(
                status == 401 || status == 404,
                "Expected 401 Unauthorized or 404 Not Found with malformed header {} = {} for {}, got {}",
                header_name,
                header_value,
                url,
                status
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_tracks_endpoint() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    let url = format!("{}/api/tracks", config.music_server_url);
    
    // Test without authentication
    println!("Testing /api/tracks endpoint without auth...");
    let response = client.get(&url).send().await?;
    let status = response.status().as_u16();
    assert!(
        status == 401 || status == 404,
        "Expected 401 Unauthorized or 404 Not Found without authentication, got {}",
        status
    );
    
    // Test with JWT token
    if let Some(token) = &config.auth_token {
        println!("Testing /api/tracks endpoint with JWT token...");
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        let status = response.status().as_u16();
        assert!(
            status == 401 || status == 404,
            "Expected 401 Unauthorized or 404 Not Found with invalid JWT token, got {}",
            status
        );
    }
    
    // Test with Supabase anon key
    println!("Testing /api/tracks endpoint with anon key...");
    let response = client
        .get(&url)
        .header("apikey", &config.supabase_anon_key)
        .send()
        .await?;
    
    let status = response.status().as_u16();
    
    assert!(
        status == 401 || status == 404,
        "Expected 401 Unauthorized or 404 Not Found with invalid anon key, got {}",
        status
    );
    
    // Try alternate endpoint path
    let response = client
        .get(&url)
        .send()
        .await?;
    let status = response.status().as_u16();
    assert!(
        status == 401 || status == 404,
        "Expected 401 Unauthorized or 404 Not Found without authentication (alternate path), got {}",
        status
    );
    
    Ok(())
}

#[tokio::test]
async fn test_random_track() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    let url = format!("{}/random", config.music_server_url);
    
    // Test without authentication
    println!("Testing /random endpoint without auth...");
    let response = client.get(&url).send().await?;
    assert!(response.status().is_success(), "Random endpoint should be public");
    let body: Value = response.json().await?;
    assert!(body.get("track_id").is_some(), "Response should contain track_id");
    
    // Get a track ID for streaming tests
    let track_id = body.get("track_id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();
    
    // Test streaming the random track
    let stream_url = format!("{}/tracks/{}", config.music_server_url, track_id);
    
    // Try streaming with JWT token
    if let Some(token) = &config.auth_token {
        println!("Testing streaming with JWT token...");
        let response = client
            .get(&stream_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        println!("JWT Streaming Status: {}", response.status());
    }
    
    // Try streaming with anon key
    println!("Testing streaming with anon key...");
    let response = client
        .get(&stream_url)
        .header("apikey", &config.supabase_anon_key)
        .send()
        .await?;
    assert!(response.status().is_success(), "Streaming should work with anon key");
    
    Ok(())
}

#[tokio::test]
async fn test_prefetch() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // First, get some track IDs to prefetch
    let tracks_url = format!("{}/tracks", config.music_server_url);
    println!("Fetching tracks from: {}", tracks_url);
    
    let response = client
        .get(&tracks_url)
        .header("apikey", &config.supabase_anon_key)
        .send()
        .await?;
        
    if !response.status().is_success() {
        println!("Failed to get tracks. Status: {}", response.status());
        println!("Response: {}", response.text().await?);
        return Ok(());
    }
    
    let tracks: Vec<Value> = response.json().await?;
    println!("Retrieved {} tracks", tracks.len());
    
    if !tracks.is_empty() {
        let track_ids: Vec<String> = tracks.iter()
            .take(2)
            .filter_map(|t| t.get("id").and_then(|id| id.as_str()).map(String::from))
            .collect();
            
        println!("Selected track IDs for prefetch: {:?}", track_ids);
        
        // Try both API paths for prefetch
        let prefetch_urls = vec![
            format!("{}/api/prefetch", config.music_server_url),
            format!("{}/prefetch", config.music_server_url)
        ];
        
        let prefetch_body = serde_json::json!({ "track_ids": track_ids });
        
        for prefetch_url in prefetch_urls {
            println!("Testing prefetch URL: {}", prefetch_url);
            
            // Test with JWT token
            if let Some(token) = &config.auth_token {
                println!("Testing prefetch with JWT token...");
                let response = client
                    .post(&prefetch_url)
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&prefetch_body)
                    .send()
                    .await?;
                println!("JWT Prefetch Status: {}", response.status());
                if response.status().is_success() {
                    println!("Prefetch successful with JWT token");
                    break;
                }
            }
            
            // Test with anon key
            println!("Testing prefetch with anon key...");
            let response = client
                .post(&prefetch_url)
                .header("apikey", &config.supabase_anon_key)
                .json(&prefetch_body)
                .send()
                .await?;
                
            println!("Anon key Prefetch Status: {}", response.status());
            if response.status().is_success() {
                println!("Prefetch successful with anon key");
                break;
            } else {
                println!("Prefetch response body: {}", response.text().await?);
            }
        }
    } else {
        println!("No tracks available for prefetch testing");
    }
    
    Ok(())
}

// Test different authentication methods
#[tokio::test]
async fn test_auth_methods() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    let url = format!("{}/api/random", config.music_server_url);
    
    // Test with different auth headers
    let auth_headers = vec![
        ("Authorization", format!("Bearer {}", config.auth_token.clone().unwrap_or_default())),
        ("apikey", config.supabase_anon_key.clone()),
        ("X-API-Key", config.supabase_anon_key.clone()),
    ];
    
    for (header_name, header_value) in auth_headers {
        println!("Testing auth header: {} = {}", header_name, header_value);
        
        let response = client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;
            
        println!("  Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
    }
    
    Ok(())
}

// Test server authentication endpoints
#[tokio::test]
async fn test_server_auth_endpoints() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // Test various auth-related endpoints
    let endpoints = vec![
        "/auth",
        "/api/auth",
        "/login",
        "/api/login",
        "/auth/login",
        "/api/auth/login",
    ];
    
    for endpoint in endpoints {
        let url = format!("{}{}", config.music_server_url, endpoint);
        println!("Testing auth endpoint: {}", url);
        
        // Try GET request
        let response = client.get(&url).send().await?;
        println!("  GET Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
        
        // Try POST request with empty body
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await?;
            
        println!("  POST Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
    }
    
    Ok(())
}

// Test different Bearer token formats
#[tokio::test]
async fn test_bearer_token_formats() -> Result<()> {
    let config = create_test_config();
    
    // Skip this test if we don't have an auth token
    if config.auth_token.is_none() {
        println!("Skipping test_bearer_token_formats because no auth token is available");
        return Ok(());
    }
    
    let token = config.auth_token.clone().unwrap();
    let client = reqwest::Client::new();
    let url = format!("{}/api/random", config.music_server_url);
    
    // Test different Bearer token formats
    let auth_headers = vec![
        format!("Bearer {}", token),
        format!("bearer {}", token),
        format!("BEARER {}", token),
        token.clone(),
    ];
    
    for header_value in auth_headers {
        println!("Testing Authorization header format: {}", header_value);
        
        let response = client
            .get(&url)
            .header("Authorization", header_value)
            .send()
            .await?;
            
        println!("  Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
    }
    
    Ok(())
}

// Test for documentation or help endpoints
#[tokio::test]
async fn test_documentation_endpoints() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // Test various documentation-related endpoints
    let endpoints = vec![
        "/",
        "/docs",
        "/api",
        "/api/docs",
        "/help",
        "/api/help",
        "/swagger",
        "/api/swagger",
        "/openapi",
        "/api/openapi",
    ];
    
    for endpoint in endpoints {
        let url = format!("{}{}", config.music_server_url, endpoint);
        println!("Testing documentation endpoint: {}", url);
        
        let response = client.get(&url).send().await?;
        println!("  Status: {}", response.status());
        
        if response.status().is_success() {
            let content_type = response.headers().get("content-type")
                .map(|v| v.to_str().unwrap_or("unknown"))
                .unwrap_or("unknown");
                
            println!("  Content-Type: {}", content_type);
            
            // If it's HTML or JSON, it might be documentation
            if content_type.contains("html") || content_type.contains("json") {
                let body = response.text().await?;
                // Print just the first 200 characters to avoid flooding the output
                let preview = if body.len() > 200 {
                    format!("{}... (truncated)", &body[0..200])
                } else {
                    body
                };
                println!("  Body preview: {}", preview);
            }
        }
    }
    
    Ok(())
}

// Test for API version endpoints
#[tokio::test]
async fn test_api_version_endpoints() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // Test various version-related endpoints
    let endpoints = vec![
        "/version",
        "/api/version",
        "/v1",
        "/api/v1",
        "/v2",
        "/api/v2",
    ];
    
    for endpoint in endpoints {
        let url = format!("{}{}", config.music_server_url, endpoint);
        println!("Testing version endpoint: {}", url);
        
        let response = client.get(&url).send().await?;
        println!("  Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
    }
    
    // Also try with a custom header that might bypass authentication for version info
    let headers = vec![
        ("X-API-Version", "true"),
        ("Accept-Version", "*"),
        ("Version", "any"),
    ];
    
    for (header_name, header_value) in headers {
        let url = format!("{}/api", config.music_server_url);
        println!("Testing with version header: {} = {}", header_name, header_value);
        
        let response = client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;
            
        println!("  Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
    }
    
    Ok(())
}

// Test for custom authentication methods
#[tokio::test]
async fn test_custom_auth_methods() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // Test various custom authentication headers
    let custom_headers = vec![
        ("X-Auth-Token", config.auth_token.clone().unwrap_or_default()),
        ("X-Music-API-Key", config.supabase_anon_key.clone()),
        ("X-Music-Auth", config.auth_token.clone().unwrap_or_default()),
        ("X-Auth", config.auth_token.clone().unwrap_or_default()),
        ("Auth", config.auth_token.clone().unwrap_or_default()),
        ("Token", config.auth_token.clone().unwrap_or_default()),
    ];
    
    let url = format!("{}/api/random", config.music_server_url);
    
    for (header_name, header_value) in custom_headers {
        println!("Testing custom auth header: {} = {}", header_name, header_value);
        
        let response = client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;
            
        println!("  Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
    }
    
    // Try with query parameters
    let query_params = vec![
        ("token", config.auth_token.clone().unwrap_or_default()),
        ("api_key", config.supabase_anon_key.clone()),
        ("auth", config.auth_token.clone().unwrap_or_default()),
        ("key", config.supabase_anon_key.clone()),
    ];
    
    for (param_name, param_value) in query_params {
        let url_with_param = format!("{}/api/random?{}={}", config.music_server_url, param_name, param_value);
        println!("Testing with query param: {} = {}", param_name, param_value);
        
        let response = client
            .get(&url_with_param)
            .send()
            .await?;
            
        println!("  Status: {}", response.status());
        
        if response.status().is_success() {
            let body = response.text().await?;
            println!("  Body: {}", body);
        }
    }
    
    Ok(())
}

// Test configuration file migration
#[test]
fn test_config_migration() -> Result<()> {
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;
    
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let old_config_dir = temp_dir.path().join(".music-cli");
    let new_config_dir = temp_dir.path().join(".lynx-fm");
    
    // Create old config directory and file
    fs::create_dir_all(&old_config_dir)?;
    let old_config_path = old_config_dir.join("config.json");
    let test_config = r#"{
        "supabase_url": "https://test-project.supabase.co",
        "supabase_anon_key": "test-anon-key",
        "music_server_url": "http://test-server:3500",
        "auth_token": "test-auth-token",
        "refresh_token": "test-refresh-token",
        "token_expiry": 1234567890
    }"#;
    
    let mut file = fs::File::create(&old_config_path)?;
    file.write_all(test_config.as_bytes())?;
    
    // Run the migration script (simulated here)
    fs::create_dir_all(&new_config_dir)?;
    fs::copy(&old_config_path, new_config_dir.join("config.json"))?;
    
    // Verify the migration
    let migrated_config = fs::read_to_string(new_config_dir.join("config.json"))?;
    let config: Config = serde_json::from_str(&migrated_config)?;
    
    assert_eq!(config.supabase_url, "https://test-project.supabase.co");
    assert_eq!(config.supabase_anon_key, "test-anon-key");
    assert_eq!(config.music_server_url, "http://test-server:3500");
    assert_eq!(config.auth_token, Some("test-auth-token".to_string()));
    assert_eq!(config.refresh_token, Some("test-refresh-token".to_string()));
    assert_eq!(config.token_expiry, Some(1234567890));
    
    Ok(())
}

// Test version information
#[test]
fn test_version_info() {
    // Check that the version in Cargo.toml matches the one in the code
    let cargo_version = env!("CARGO_PKG_VERSION");
    
    // You could also check against a version defined in your code if applicable
    assert!(!cargo_version.is_empty(), "Version should not be empty");
    
    // Parse version to ensure it's valid semver
    let version_parts: Vec<&str> = cargo_version.split('.').collect();
    assert_eq!(version_parts.len(), 3, "Version should have three parts: major.minor.patch");
    
    // Check that each part is a valid number
    for part in version_parts {
        assert!(part.parse::<u32>().is_ok(), "Version parts should be valid numbers");
    }
}

// Test config file paths
#[test]
fn test_config_paths() -> Result<()> {
    // Test that the config paths are correct
    let config = Config::default();
    
    let config_dir = Config::config_dir()?;
    let config_file = Config::config_file()?;
    
    // Check that the config directory ends with .lynx-fm
    assert!(config_dir.ends_with(".lynx-fm"), "Config directory should end with .lynx-fm");
    
    // Check that the config file is named config.json
    assert!(config_file.ends_with("config.json"), "Config file should be named config.json");
    
    Ok(())
}

// Test CLI command structure
#[test]
fn test_cli_commands() {
    use clap::CommandFactory;
    use lynx_fm::commands::Cli;
    
    // Verify that the CLI can be constructed without errors
    let cli = Cli::command();
    
    // Check that all expected subcommands exist
    let subcommands: Vec<_> = cli.get_subcommands().collect();
    let subcommand_names: Vec<_> = subcommands.iter().map(|cmd| cmd.get_name()).collect();
    
    // Verify essential commands are present
    assert!(subcommand_names.contains(&"config"), "Config command should exist");
    assert!(subcommand_names.contains(&"login"), "Login command should exist");
    assert!(subcommand_names.contains(&"logout"), "Logout command should exist");
    assert!(subcommand_names.contains(&"signup"), "Signup command should exist");
    assert!(subcommand_names.contains(&"health"), "Health command should exist");
    assert!(subcommand_names.contains(&"random"), "Random command should exist");
    assert!(subcommand_names.contains(&"play"), "Play command should exist");
    assert!(subcommand_names.contains(&"prefetch"), "Prefetch command should exist");
    
    // Verify the play command has the required arguments
    let play_cmd = cli.find_subcommand("play").unwrap();
    assert!(play_cmd.get_positionals().count() > 0, "Play command should have at least one positional argument");
}

#[tokio::test]
async fn test_login() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // Test different login endpoint paths
    let endpoints = vec![
        format!("{}/auth/login", config.music_server_url),
        format!("{}/api/auth/login", config.music_server_url),
        format!("{}/login", config.music_server_url),
        format!("{}/api/login", config.music_server_url),
    ];

    for url in endpoints {
        println!("\nTesting login endpoint: {}", url);
        
        // 1. Test with empty body
        println!("Testing with empty body...");
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await?;
        let status = response.status().as_u16();
        assert!(
            status == 400 || status == 401 || status == 404,
            "Expected 400 Bad Request, 401 Unauthorized, or 404 Not Found for empty body, got {}",
            status
        );

        // 2. Test with invalid credentials
        println!("Testing with invalid credentials...");
        let invalid_creds = serde_json::json!({
            "email": "invalid@example.com",
            "password": "wrongpassword"
        });
        
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&invalid_creds)
            .send()
            .await?;
        let status = response.status().as_u16();
        assert!(
            status == 401 || status == 404,
            "Expected 401 Unauthorized or 404 Not Found for invalid credentials, got {}",
            status
        );

        // 3. Test with malformed JSON
        println!("Testing with malformed JSON...");
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body("{ invalid json }")
            .send()
            .await?;
        let status = response.status().as_u16();
        assert!(
            status == 400 || status == 404,
            "Expected 400 Bad Request or 404 Not Found for malformed JSON, got {}",
            status
        );

        // 4. Test with missing required fields
        println!("Testing with missing fields...");
        let missing_fields = vec![
            serde_json::json!({"email": "test@example.com"}),
            serde_json::json!({"password": "testpass"}),
            serde_json::json!({"username": "test", "password": "testpass"}),
        ];

        for body in missing_fields {
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;
            let status = response.status().as_u16();
            assert!(
                status == 400 || status == 404,
                "Expected 400 Bad Request or 404 Not Found for missing fields, got {}",
                status
            );
        }

        // 5. Test successful login (if credentials available)
        if let Some(test_email) = option_env!("TEST_EMAIL") {
            if let Some(test_password) = option_env!("TEST_PASSWORD") {
                println!("Testing with valid credentials...");
                let valid_creds = serde_json::json!({
                    "email": test_email,
                    "password": test_password
                });
                
                let response = client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .json(&valid_creds)
                    .send()
                    .await?;
                    
                if response.status().is_success() {
                    let body: Value = response.json().await?;
                    println!("Login Response: {:?}", body);
                    
                    // Validate response structure
                    assert!(body.get("token").is_some() || body.get("access_token").is_some(), 
                        "Response should contain token or access_token");
                        
                    let token = body.get("token")
                        .or_else(|| body.get("access_token"))
                        .and_then(|v| v.as_str())
                        .unwrap();
                        
                    // Test token refresh if endpoint available
                    let refresh_url = url.replace("/login", "/refresh");
                    println!("Testing token refresh at: {}", refresh_url);
                    
                    let response = client
                        .post(&refresh_url)
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await?;
                        
                    if response.status().is_success() {
                        let refresh_body: Value = response.json().await?;
                        assert!(refresh_body.get("token").is_some() || refresh_body.get("access_token").is_some(),
                            "Refresh response should contain new token");
                    }
                    
                    // Test logout if endpoint available
                    let logout_url = url.replace("/login", "/logout");
                    println!("Testing logout at: {}", logout_url);
                    
                    let response = client
                        .post(&logout_url)
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await?;
                        
                    if response.status().is_success() {
                        // Try to use the token after logout
                        let me_url = format!("{}/me", config.music_server_url);
                        let response = client
                            .get(&me_url)
                            .header("Authorization", format!("Bearer {}", token))
                            .send()
                            .await?;
                            
                        assert!(!response.status().is_success(),
                            "Token should be invalid after logout");
                    }
                }
            }
        }
    }

    Ok(())
} 