# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK="testnet"

echo "Build and optimize the contracts..."

# make build >/dev/null
task build
cd target/wasm32v1-none/release

echo "Contracts compiled."
echo "Optimize contracts..."

soroban contract optimize --wasm soroban_token_contract.wasm
soroban contract optimize --wasm index.wasm
soroban contract optimize --wasm index_factory.wasm

echo "Contracts optimized."

# # Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)

echo "Deploy the soroban_token_contract and capture its contract ID hash..."

XLM="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

echo "Install the soroban_token and pool contracts..."

TOKEN_WASM_HASH=$(soroban contract upload \
    --wasm soroban_token_contract.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

# Continue with the rest of the deployments
INDEX_WASM_HASH=$(soroban contract upload \
    --wasm index.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

echo "Token and index contracts deployed."

#   _______   __       ______  ___________  ______     _______   ___  ___
#  /"     "| /""\     /" _  "\("     _   ")/    " \   /"      \ |"  \/"  |
# (: ______)/    \   (: ( \___))__/  \\__/// ____  \ |:        | \   \  /
#  \/    | /' /\  \   \/ \        \\_ /  /  /    ) :)|_____/   )  \\  \/
#  // ___)//  __'  \  //  \ _     |.  | (: (____/ //  //      /   /   /
# (:  (  /   /  \\  \(:   _) \    \:  |  \        /  |:  __   \  /   /
#  \__/ (___/    \___)\_______)    \__|   \"_____/   |__|  \___)|___/

echo "Initialize index factory..."

INDEX_FACTORY_ADDR=$(soroban contract deploy \
    --wasm index_factory.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK)

stellar contract invoke \
    --id $INDEX_FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    init_admin \
    --account $ADMIN_ADDRESS

stellar contract invoke \
    --id $INDEX_FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    set_pool_hash \
    --admin $ADMIN_ADDRESS \
    --new_hash $POOL_WASM_HASH

stellar contract invoke \
    --id $INDEX_FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    set_token_hash \
    --admin $ADMIN_ADDRESS \
    --new_hash $TOKEN_WASM_HASH

echo "Initialize index through factory..."

stellar contract invoke \
    --id $INDEX_FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    deploy_index_contract \
    --user $ADMIN_ADDRESS \
    --oracle_registry_ids '["BTC", "XLM"]' \
    --asset CAVLP5DH2GJPZMVO7IJY4CVOD5MWEFTJFVPD2YY2FQXOQHRGHK4D6HLP \
    --tokens "[\"$nBTC_TOKEN_ADDR\", \"$XLM\"]" \
    --lp_token_info '["Pool Share Token", "POOL"]' \
    --fee_fraction 30 \
    --tier '"A"' \
    --quote_max_insurance 1000000 \
    --oracle_registry $ORACLE_REGISTRY_ADDR

echo "Query index address..."

INDEX_ADDR=$(soroban contract invoke \
    --id $POOL_ROUTER_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 100 \
    -- \
    query_pools | jq -r '.[0]')

echo "Index contract initialized."

echo "Mint XLM token to the admin and provide liquidity..."

soroban contract invoke \
    --id $XLM \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    -- \
    mint --to $ADMIN_ADDRESS --amount 10000000000 # 7 decimals, 10k tokens

# Provide liquidity to the pool
soroban contract invoke \
    --id $INDEX_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK --fee 10000000 \
    -- \
    mint --user $ADMIN_ADDRESS --amount 6000000000

echo "Index tokens minted."

echo "#############################"

echo "Initialization complete!"
echo "XLM address: $XLM"

echo "Inde Factory Contract address: $INDEX_FACTORY_ADDR"
echo "Index Contract address: $INDEX_ADDR"
