# Music CLI Tests

This directory contains integration tests for the Music CLI application, focusing on testing the interaction with the music server.

## Test Results Summary

Based on our tests, we've learned the following about the music server:

1. The server's `/health` endpoint is accessible without authentication and returns a 200 OK status.
2. All other endpoints, including `/api/random`, require authentication and return a 401 Unauthorized status.
3. We've tried various authentication methods (Bearer token, apikey, X-API-Key, custom headers, query parameters) and formats, but none of them work.
4. We've also tried various authentication endpoints, but none of them seem to be available.

## Authentication Issues

The main issue appears to be with authentication. The server consistently returns a 401 Unauthorized status for all endpoints except `/health`. We've tried:

- Standard Bearer token authentication
- Different Bearer token formats (with/without "Bearer" prefix, different casing)
- Supabase anon key as an API key
- Custom authentication headers (X-Auth-Token, X-Music-API-Key, etc.)
- Query parameters for authentication (token, api_key, auth, key)

None of these methods have been successful.

## Recommendations

1. **Contact the server administrator**: The most direct solution would be to contact the administrator of the music server to get information about the required authentication method.

2. **Check server logs**: If you have access to the server logs, check them when making requests to see what authentication errors are being logged.

3. **Try a different approach**: Instead of using the Supabase JWT token, try creating a custom token or using a different authentication method.

4. **Implement a fallback mode**: Modify the Music CLI to work in a "local-only" mode that doesn't require authentication, if possible.

5. **Check for server updates**: The server might have been updated with new authentication requirements. Check if there's a newer version of the server documentation.

6. **Inspect network traffic**: Use a tool like Wireshark or the browser's network inspector to capture successful authentication requests from other clients.

## Running the Tests

To run all tests:

```bash
cargo test -- --nocapture
```

To run a specific test:

```bash
cargo test test_health_check -- --nocapture
```

## Test Descriptions

- `test_health_check`: Tests the health check endpoint of the music server.
- `test_random_track_without_auth`: Tests retrieving a random track without authentication.
- `test_random_track_with_auth`: Tests retrieving a random track with authentication.
- `test_api_endpoints`: Tests various API endpoints on the music server.
- `test_auth_methods`: Tests different authentication methods.
- `test_server_auth_endpoints`: Tests various authentication-related endpoints.
- `test_documentation_endpoints`: Tests for documentation or help endpoints.
- `test_api_version_endpoints`: Tests for API version information.
- `test_custom_auth_methods`: Tests custom authentication methods and formats.

## Next Steps

Based on the test results, the next step would be to investigate the server's authentication requirements further or to implement a workaround that doesn't rely on the server's authentication. 