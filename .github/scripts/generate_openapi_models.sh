#!/bin/bash

# Move to the root directory
cd ../..
# Ensure openapi-generator-cli is installed
if ! command -v openapi-generator-cli &> /dev/null; then
    echo "Error: openapi-generator-cli is not installed. Please install it."
    exit 1
fi

# Ensure openapi.yaml exists
if [ ! -f "openapi.yaml" ]; then
    echo "Error: openapi.yaml not found. Please place your OpenAPI specification file in the root directory of your project."
    exit 1
fi

# Ensure Rust is selected as the generator
GENERATOR="rust"

# Ensure the new directory is inside api directory

# Ensure output directory is specified
OUTPUT_DIR="openapi"

# Generate models
openapi-generator-cli generate -i openapi.yaml -g $GENERATOR -o $OUTPUT_DIR


# Move the openapi folder into the api folder
mv openapi api/

