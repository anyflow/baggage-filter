FROM scratch
COPY target/wasm32-unknown-unknown/release/baggage_filter.optimized.wasm /plugin.wasm
