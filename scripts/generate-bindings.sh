# Create bindings directory
mkdir -p bindings

# Generate bindings for each contract
soroban contract bindings typescript --wasm target/wasm32v1-none/release/adapter_registry.wasm --output-dir bindings/adapter_registry --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/normal_adapter.wasm --output-dir bindings/normal_adapter --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/aquarius_adapter.wasm --output-dir bindings/aquarius_adapter --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/soroswap_adapter.wasm --output-dir bindings/soroswap_adapter --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/index_fund.wasm --output-dir bindings/index_fund --overwrite

soroban contract bindings typescript --wasm target/wasm32v1-none/release/index_fund_factory.wasm --output-dir bindings/index_fund_factory --overwrite