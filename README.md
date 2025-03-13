# Lynx.fm CLI

A command-line interface for interacting with the Lynx.fm music streaming service.

## Features

- Authentication with Supabase (signup, login, logout)
- Email verification during signup
- Play random tracks
- Stream specific tracks
- Prefetch tracks for faster playback
- Health check for the server

## Installation

### Option 1: Homebrew (macOS)

```bash
brew install cli-lynx-fm
```

### Option 2: Cargo (Rust's package manager)

```bash
cargo install cli-lynx-fm
```

### Option 3: Docker

```bash
docker pull yourusername/cli-lynx-fm
docker run -it --rm yourusername/cli-lynx-fm --help
```

### Option 4: Building from source

```bash
git clone https://github.com/yourusername/cli-lynx-fm.git
cd cli-lynx-fm
cargo build --release
```

The binary will be available at `target/release/lynx-fm`.

## Configuration

Before using the CLI, you need to configure it with your Supabase and server URLs:

```bash
# Set Supabase URL and anonymous key
lynx-fm config --supabase-url https://your-project.supabase.co --supabase-key your-anon-key

# Set music server URL (if different from default)
lynx-fm config --server-url https://server.lg.media
```

When using Docker, you can mount a configuration volume:

```bash
docker run -it --rm -v ~/.lynx-fm:/root/.lynx-fm yourusername/cli-lynx-fm config --supabase-url https://your-project.supabase.co --supabase-key your-anon-key
```

## Usage

### Authentication

```bash
# Sign up for a new account
lynx-fm signup

# Log in to your account
lynx-fm login

# Log out from your account
lynx-fm logout
```

During signup, you'll be prompted to enter your email and password. After submitting, you'll receive a verification code via email that you'll need to enter to complete the signup process.

### Playing Music

```bash
# Play a random track
lynx-fm random

# Play a specific track
lynx-fm play track_id

# Prefetch tracks for faster playback
lynx-fm prefetch track_id1 track_id2 track_id3
```

### Server Health Check

```bash
# Check if the server is healthy
lynx-fm health
```

## How It Works

1. **Authentication**: The CLI uses Supabase for authentication, storing your JWT token securely in a config file.
2. **Token Management**: Tokens are automatically refreshed when needed.
3. **Music Streaming**: When playing a track, the CLI streams the audio data and plays it through your default audio device.
4. **Prefetching**: You can prefetch tracks to improve playback performance.

## Configuration File

The CLI stores its configuration in `~/.lynx-fm/config.json`. This includes:

- Supabase URL and anonymous key
- Music server URL
- Authentication tokens (if logged in)

## Development

### Project Structure

- `src/main.rs`: Entry point and command handling
- `src/auth.rs`: Authentication with Supabase
- `src/music.rs`: Interaction with the music server
- `src/config.rs`: Configuration management
- `src/commands.rs`: CLI command definitions
- `tests/`: Integration tests for the Lynx.fm CLI

### Adding New Features

To add a new command:

1. Add it to the `Commands` enum in `src/commands.rs`
2. Implement the command handler in `src/main.rs`
3. Add any necessary client methods in `src/music.rs` or `src/auth.rs`

## Deployment

This project is set up with automated deployment pipelines to publish to multiple platforms:

### Release Process

To release a new version:

1. Run the release script with the new version number:
   ```bash
   ./scripts/release.sh 0.1.1
   ```

2. The script will:
   - Update the version in Cargo.toml
   - Build and test the project
   - Commit the changes
   - Create and push a git tag (e.g., v0.1.1)
   - Push the changes to GitHub

3. The GitHub Actions workflow will automatically:
   - Build and test the application
   - Publish to crates.io
   - Build and push Docker images to Docker Hub
   - Update the Homebrew formula

### Required GitHub Secrets

For the automated deployment to work, the following secrets must be set in the GitHub repository:

- `CRATES_IO_TOKEN`: Your crates.io API token with `publish-new` and `publish-update` scopes
- `DOCKERHUB_USERNAME`: Your Docker Hub username
- `DOCKERHUB_TOKEN`: Your Docker Hub access token with Read & Write permissions

### Deployment Files Structure

The following files are used for the deployment process:

- `.github/workflows/release.yml`: GitHub Actions workflow for automated deployment
- `homebrew/cli-lynx-fm.rb`: Homebrew formula template
- `Dockerfile`: Docker image definition
- `scripts/release.sh`: Script to automate the release process
- `.dockerignore`: Files to exclude from Docker builds

### GitHub Actions Workflow

The GitHub Actions workflow in `.github/workflows/release.yml` is triggered when a new tag is pushed to the repository. It consists of the following jobs:

1. **Build**: Builds and tests the application, uploads the binary as an artifact
2. **Publish to crates.io**: Publishes the package to crates.io
3. **Publish to Docker Hub**: Builds and pushes the Docker image to Docker Hub
4. **Update Homebrew Formula**: Updates the Homebrew formula with the new version and SHA256 hash

### Deployment Platforms

#### Homebrew (macOS)

The Homebrew formula is located in `homebrew/cli-lynx-fm.rb`. When a new version is released, the GitHub Actions workflow updates the formula with the new version and SHA256 hash.

#### Crates.io (Rust's package manager)

The package metadata is defined in `Cargo.toml`. The GitHub Actions workflow publishes the package to crates.io using the provided API token.

#### Docker Hub

The Docker image is built using the `Dockerfile` in the root directory. The GitHub Actions workflow builds and pushes the image to Docker Hub with appropriate tags.

### Migrating from music-cli

If you're migrating from the previous `music-cli` application, you can use the provided migration script:

```bash
./migrate_config.sh
```

This will copy your existing configuration from `~/.music-cli/` to `~/.lynx-fm/`.

## Known Issues

### Authentication

The CLI currently has issues authenticating with the music server. Our tests have shown that:

1. The server's `/health` endpoint is accessible without authentication.
2. All other endpoints, including `/api/random`, require authentication and return a 401 Unauthorized status.
3. We've tried various authentication methods (Bearer token, apikey, X-API-Key, custom headers, query parameters) and formats, but none of them work.

If you're experiencing authentication issues, please:

1. Check that your Supabase URL and anonymous key are correctly configured.
2. Ensure you're logged in (`lynx-fm login`).
3. Verify that the music server URL is correct (`lynx-fm config`).
4. Contact the server administrator for the correct authentication method.

See the [tests/README.md](tests/README.md) file for more details on our authentication testing.

## Testing

To run the tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_health_check -- --nocapture
```

The tests in the `tests/` directory are designed to help diagnose issues with the music server connection and authentication, as well as verify the functionality of the CLI. The test suite includes:

1. **API Tests**: Tests for interacting with the music server API
   - Health check
   - Random track retrieval
   - Authentication methods

2. **Configuration Tests**: Tests for configuration management
   - Config file paths
   - Config migration from music-cli to lynx-fm

3. **CLI Structure Tests**: Tests to ensure the CLI commands are properly defined
   - Command existence
   - Required arguments

4. **Version Tests**: Tests to ensure version information is correctly defined
   - Semantic versioning format
   - Version consistency

These tests help ensure that the CLI functions correctly and can handle various server configurations and authentication methods.

## License

[MIT License](LICENSE) 