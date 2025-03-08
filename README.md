# `baggage-filter`

## Introduction

A Rust-based Istio WASM filter that generates a W3C Trace Context-compliant `baggage` header, containing key-value pairs specified by the configuration.

## Features

- Adds the headers specified in the config as items in the `baggage` header. If the header does not exist, create it. If it already exists, overwrite the value of keys with the same name while retaining the other items. Refer to [`./resources/wasmplugin.yaml`](./resources/wasmplugin.yaml).

## Getting started

```shell
# Install Rust. Note: Installing via brew on macOS may not compile correctly. Follow the official Rust installation path instead.
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-make (build tool; refer to Makefile.toml).
> cargo install cargo-make

# Install wasm-opt (binaryen) (for macOS; other OSes require a different method. If installation fails, skip this step by removing the optimize-wasm task from Makefile.toml).
> brew install binaryen

# Create a .env file at the root and set DOCKER_IMAGE_PATH. Example below:
DOCKER_IMAGE_PATH=anyflow/baggage-filter

# Run tests -> Rust build -> Image optimization -> Docker build -> Docker push
> cargo make deploy
```

## How to Test at Runtime in Istio

```shell
# Change the WASM log level of the target pod to debug.
> istioctl pc log -n <namespace name> <pod name> --level wasm:debug

# Filter logs to show only baggage-filter
> k logs -n <namespace name> <pod name> -f | grep -F '[bf]'

# Apply resources/telemetry.yaml: To add baggage header in the trace tag.
> kubectl apply -f telemetry.yaml

# Apply resources/wasmplugin.yaml: Check logs to confirm successful loading, e.g., "[bf] Configuration successful, configured headers:".
> kubectl apply -f wasmplugin.yaml

# Make a curl request and verify if the matching success log appears, e.g., "[bf] Set new baggage: x-message-id=0123456789".
```

## License

`baggage-filter` is released under version 2.0 of the Apache License.
