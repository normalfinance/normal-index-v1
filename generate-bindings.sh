# Create bindings directory
mkdir -p bindings

# Generate bindings for each contract
soroban contract bindings typescript --wasm target/wasm32v1-none/release/index_token.wasm --output-dir bindings/index_token --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/index.wasm --output-dir bindings/index --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/index_factory.wasm --output-dir bindings/index_factory --overwrite