use anyhow::Result;
use reqwest::StatusCode;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    supabase_url: String,
    supabase_anon_key: String,
    music_server_url: String,
    auth_token: Option<String>,
    refresh_token: Option<String>,
    token_expiry: Option<i64>,
}

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

/// Test JWT validation with various scenarios
#[tokio::test]
async fn test_jwt_validation() -> Result<()> {
    let config = create_test_config();
    let client = reqwest::Client::new();
    
    // Test endpoints that should require authentication
    let protected_endpoints = vec![
        "/me",
        "/api/me",
        "/api/tracks",
        "/api/playlists",
    ];

    println!("\n=== JWT Validation Test Cases ===\n");

    for endpoint in &protected_endpoints {
        let url = format!("{}{}", config.music_server_url, endpoint);
        println!("\nTesting endpoint: {}", url);

        // Case 1: No Authentication
        println!("\n1. Testing with no authentication...");
        let response = client.get(&url).send().await?;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 Unauthorized without authentication"
        );
        let error_body = response.text().await?;
        println!("Response (no auth): {}", error_body);

        // Case 2: Invalid JWT Format
        println!("\n2. Testing with invalid JWT format...");
        let response = client
            .get(&url)
            .header("Authorization", "Bearer invalid.jwt.token")
            .send()
            .await?;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 Unauthorized with invalid JWT format"
        );
        let error_body = response.text().await?;
        println!("Response (invalid format): {}", error_body);

        // Case 3: Expired JWT
        println!("\n3. Testing with expired JWT...");
        let expired_jwt = create_test_jwt(
            &config.supabase_url,
            "test@example.com",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs() as i64 - 3600,
        )?;
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", expired_jwt))
            .send()
            .await?;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 Unauthorized with expired JWT"
        );
        let error_body = response.text().await?;
        println!("Response (expired): {}", error_body);

        // Case 4: Valid JWT
        if let Some(token) = &config.auth_token {
            println!("\n4. Testing with valid JWT...");
            let response = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Expected 200 OK with valid JWT"
            );
            let body = response.text().await?;
            println!("Response (valid): {}", body);
        }

        // Case 5: Supabase Anonymous Key
        println!("\n5. Testing with Supabase anon key...");
        let response = client
            .get(&url)
            .header("apikey", &config.supabase_anon_key)
            .send()
            .await?;
        if response.status().is_success() {
            let body = response.text().await?;
            println!("Response (anon key): {}", body);
            assert!(
                body.contains("anon-user"),
                "Anonymous access should identify as 'anon-user'"
            );
        } else {
            println!("Anonymous access not allowed for this endpoint (expected)");
        }
    }

    // Test JWT Claims Validation
    println!("\n=== Testing JWT Claims Validation ===\n");
    let test_cases = vec![
        ("Missing 'sub' claim", json!({
            "iss": config.supabase_url,
            "aud": "authenticated",
            "exp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64 + 3600,
            "email": "test@example.com"
        })),
        ("Missing 'aud' claim", json!({
            "iss": config.supabase_url,
            "sub": "test-user-id",
            "exp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64 + 3600,
            "email": "test@example.com"
        })),
        ("Wrong issuer", json!({
            "iss": "https://wrong-issuer.com",
            "sub": "test-user-id",
            "aud": "authenticated",
            "exp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64 + 3600,
            "email": "test@example.com"
        })),
        ("Wrong audience", json!({
            "iss": config.supabase_url,
            "sub": "test-user-id",
            "aud": "wrong-audience",
            "exp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64 + 3600,
            "email": "test@example.com"
        }))
    ];

    for (test_name, claims) in test_cases {
        println!("\nTesting {}", test_name);
        let test_jwt = create_test_jwt_with_claims(&claims)?;
        let url = format!("{}/me", config.music_server_url);
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", test_jwt))
            .send()
            .await?;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 Unauthorized for {}",
            test_name
        );
        let error_body = response.text().await?;
        println!("Response: {}", error_body);
    }

    Ok(())
}

// Helper function to create a test JWT with custom claims
fn create_test_jwt_with_claims(claims: &serde_json::Value) -> Result<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let key = EncodingKey::from_secret(b"test-secret");
    let token = encode(&Header::default(), claims, &key)?;
    Ok(token)
}

// Helper function to create a test JWT with basic claims
fn create_test_jwt(issuer: &str, email: &str, exp: i64) -> Result<String> {
    let claims = json!({
        "iss": issuer,
        "sub": "test-user-id",
        "aud": "authenticated",
        "exp": exp,
        "email": email,
        "role": "authenticated"
    });
    create_test_jwt_with_claims(&claims)
} 