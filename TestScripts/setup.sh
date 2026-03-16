#!/bin/bash

DFX_VERSION=0.31.0

echo Checking Environment for InheritNext

if ! command -v dfx >/dev/null 2>&1; then
    echo DFX not found
    echo Install with: sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
    exit 1
fi

version=$(dfx --version | awk '{print $2}')
if [ "$version" != "$DFX_VERSION" ]; then
    echo DFX version mismatch
    echo Required: $DFX_VERSION
    echo Installed: $version
    exit 1
fi

if ! command -v rustc >/dev/null 2>&1; then
    echo Rust not found
    echo Install with: curl https://sh.rustup.rs -sSf | sh
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo Cargo not found
    exit 1
fi

if ! command -v candid-extractor >/dev/null 2>&1; then
    echo candid-extractor not found
    echo Install with: cargo install candid-extractor
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.." || exit 1

cargo build
cargo install generate-did
rustup target add wasm32-unknown-unknown
npm install

$SCRIPT_DIR/startDev.sh

# echo "Run: TestScripts/startDev.sh"
