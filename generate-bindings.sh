# Create bindings directory
mkdir -p bindings

# Generate bindings for each contract
soroban contract bindings typescript --wasm target/wasm32v1-none/release/index.wasm --output-dir bindings/index

soroban contract bindings typescript --wasm target/wasm32v1-none/release/index_factory.wasm --output-dir bindings/index_factory

soroban contract bindings typescript --wasm target/wasm32v1-none/release/soroban_token_contract.wasm --output-dir bindings/soroban_token_contract