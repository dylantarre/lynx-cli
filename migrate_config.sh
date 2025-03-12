#!/bin/bash

# Migration script to move from music-cli to lynx-fm

OLD_CONFIG_DIR="$HOME/.music-cli"
NEW_CONFIG_DIR="$HOME/.lynx-fm"

# Check if old config directory exists
if [ -d "$OLD_CONFIG_DIR" ]; then
    echo "Found existing music-cli configuration at $OLD_CONFIG_DIR"
    
    # Create new config directory if it doesn't exist
    mkdir -p "$NEW_CONFIG_DIR"
    
    # Copy configuration files
    if [ -f "$OLD_CONFIG_DIR/config.json" ]; then
        echo "Migrating configuration..."
        cp "$OLD_CONFIG_DIR/config.json" "$NEW_CONFIG_DIR/config.json"
        echo "Configuration migrated successfully!"
    fi
    
    echo "Migration complete! You can now use lynx-fm."
    echo "The old configuration at $OLD_CONFIG_DIR has been preserved."
    echo "You can remove it manually once you've confirmed everything works."
else
    echo "No existing music-cli configuration found. Starting fresh with lynx-fm."
    mkdir -p "$NEW_CONFIG_DIR"
fi 