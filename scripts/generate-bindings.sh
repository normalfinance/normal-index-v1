# Create bindings directory
mkdir -p bindings

# Generate bindings for each contract
soroban contract bindings typescript --wasm target/wasm32v1-none/release/swap_utility.wasm --output-dir bindings/swap_utility --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/index_fund.wasm --output-dir bindings/index_fund --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/index_factory.wasm --output-dir bindings/index_factory --overwrite