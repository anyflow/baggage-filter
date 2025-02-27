# baggage-filter

## Introduction

A Rust-based Istio WASM filter that create the `baggage` header which has header key and value pairs designated by configuration. `baggage` header is the one of W3C trace context.

## Getting started

```shell
# Rust 설치. 참고로 macOS에서 brew로 설치하면 정상 compile안됨. 따라서 Rust 공식 설치 Path를 따라야.
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# cargo-make 설치 (빌드 도구. Makefile.toml 참고)
> cargo install cargo-make

# wasm-opt (bynaryen) 설치 (macOS의 경우. 타 OS의 경우 별도 방법 필요. 설치 안될 경우 Makefile.toml의 optimize-wasm task 제거로 본 step skip 가능)
> brew install binaryen

# test -> rust build -> image optimization -> docker build -> docker push
> cargo make clean-all
```