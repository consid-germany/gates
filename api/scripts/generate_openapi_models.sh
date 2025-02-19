#!/bin/bash

# Navigate to the root directory of the project
pushd "$(git rev-parse --show-toplevel)" || exit

# Ensure openapi.yaml exists
if [ ! -f "openapi.yaml" ]; then
    echo "Error: openapi.yaml not found. Please place your OpenAPI specification file in the root directory of your project."
    exit 1
fi

# Rust is selected as the generator
GENERATOR="rust"

# Ensure output directory is specified
OUTPUT_DIR="/api/openapi"

# Uncomment the line below to pull the docker image locally
# docker pull openapitools/openapi-generator-cli

# Uncomment the line below to Generate models using the openapi-generator-cli locally
# openapi-generator-cli generate -i openapi.yaml -g $GENERATOR -o $OUTPUT_DIR

# Generate openapi models using the docker image
docker run --rm \
  -v ${PWD}:/local openapitools/openapi-generator-cli generate \
  -i /local/openapi.yaml \
  --additional-properties=avoidBoxedModels=true \
  -g $GENERATOR \
  -o /local/$OUTPUT_DIR

popd