#!/bin/bash

# Check if yq is installed
if ! command -v yq &> /dev/null; then
    echo "yq is required but not installed. Please install it first:"
    echo "brew install yq  # for macOS"
    echo "or visit https://github.com/mikefarah/yq#install for other platforms"
    exit 1
fi

# Default values
CONFIG_FILE="config/config.dev.toml"
ENV_FILE=".env"

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --config) CONFIG_FILE="$2"; shift ;;
        --env) ENV_FILE="$2"; shift ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Convert TOML to YAML and process each line
# Only convert keys to uppercase, keep values as is
yq -o=props "$CONFIG_FILE" | while IFS='=' read -r key value; do
    # Convert dots to underscores and uppercase only the key
    key=$(echo "$key" | sed 's/\./\_/g' | tr '[:lower:]' '[:upper:]')
    # Print the uppercase key with original value
    echo "$key=$value"
done > "$ENV_FILE"

echo "Environment variables written to $ENV_FILE"

# Display the variables (without sensitive information)
echo "Generated environment variables (excluding passwords):"
grep -v "PASSWORD\|SECRET\|KEY" "$ENV_FILE" || true
