#!/bin/bash

# Uncomment the line below to run the script locally
#cd ../..

# Check if Java is installed
if ! command -v java &> /dev/null; then
    echo "Java Development Kit (JDK) is not installed. Please install JDK."
    # Provide instructions on how to install Java
    exit 1
fi
# Check if openapi-generator-cli is installed
if ! command -v openapi-generator &> /dev/null; then
    echo "openapi-generator-cli is not installed. Installing..."
    npm install @openapitools/openapi-generator-cli -g
fi

# Ensure openapi.yaml exists
if [ ! -f "openapi.yaml" ]; then
    echo "Error: openapi.yaml not found. Please place your OpenAPI specification file in the root directory of your project."
    exit 1
fi

# Rust is selected as the generator
GENERATOR="rust"

# Ensure output directory is specified
OUTPUT_DIR="openapi"

# Generate models
openapi-generator-cli generate -i openapi.yaml -g $GENERATOR -o $OUTPUT_DIR


# Move the openapi folder into the api folder
mv openapi api/

