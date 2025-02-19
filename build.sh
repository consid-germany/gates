#!/bin/bash

pushd api
. ./scripts/generate_openapi_models.sh
cargo lambda build --arm64 --release
popd

pushd ui
npm ci
echo "PUBLIC_API_BASE_URL=/api" > .env
npm run build
popd

pushd action
npm ci
npm run build
popd

pushd cdk
npm ci
npm run build
npm run package
popd
