# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <identity_string> <network>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK=$2

# Load env vars dynamically
source "$(dirname "${BASH_SOURCE[0]}")/load-env.sh" "$NETWORK"

echo "Build and optimize the contracts..."

# make build >/dev/null
task build
cd target/wasm32v1-none/release

echo "Contracts compiled."
echo "Optimize contracts..."

soroban contract optimize --wasm swap_utility.wasm
soroban contract optimize --wasm index.wasm
soroban contract optimize --wasm index_factory.wasm

echo "Contracts optimized."

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)

#   __    _____  ___   ________    _______  ___  ___  
#  |" \  (\"   \|"  \ |"      "\  /"     "||"  \/"  | 
#  ||  | |.\\   \    |(.  ___  :)(: ______) \   \  /  
#  |:  | |: \.   \\  ||: \   ) || \/    |    \\  \/   
#  |.  | |.  \    \. |(| (___\ || // ___)_   /\.  \   
#  /\  |\|    \    \ ||:       :)(:      "| /  \   \  
# (__\_|_)\___|\____\)(________/  \_______)|___/\___| 
                                                    
echo "Install the Index contract..."

INDEX_WASM_HASH=$(soroban contract upload \
    --wasm index.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE)

echo "Index contract deployed."

#   ________  __   __  ___       __         _______   
#  /"       )|"  |/  \|  "|     /""\       |   __ "\  
# (:   \___/ |'  /    \:  |    /    \      (. |__) :) 
#  \___  \   |: /'        |   /' /\  \     |:  ____/  
#   __/  \\   \//  /\'    |  //  __'  \    (|  /      
#  /" \   :)  /   /  \\   | /   /  \\  \  /|__/ \     
# (_______/  |___/    \___|(___/    \___)(_______)                    

echo "Deploy the Swap Utility..."

SWAP_UTILITY_ADDR=$(soroban contract deploy \
    --wasm swap_utility.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE)

#   _______   __       ______  ___________  ______     _______   ___  ___
#  /"     "| /""\     /" _  "\("     _   ")/    " \   /"      \ |"  \/"  |
# (: ______)/    \   (: ( \___))__/  \\__/// ____  \ |:        | \   \  /
#  \/    | /' /\  \   \/ \        \\_ /  /  /    ) :)|_____/   )  \\  \/
#  // ___)//  __'  \  //  \ _     |.  | (: (____/ //  //      /   /   /
# (:  (  /   /  \\  \(:   _) \    \:  |  \        /  |:  __   \  /   /
#  \__/ (___/    \___)\_______)    \__|   \"_____/   |__|  \___)|___/

echo "Deploy the Index Factory..."

INDEX_FACTORY_ADDR=$(soroban contract deploy \
    --wasm index_factory.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    --admin $ADMIN_ADDRESS \
    --emergency_admin \
    --swap_utility $SWAP_UTILITY_ADDR \
    --index_contract_wasm $INDEX_WASM_HASH \
    --max_manager_fee_amount 10 \
    --protocol_fee_amount 10 \
    --protocol_fee_recipient $ADMIN_ADDRESS \
    --minimum_fee_threshold 10)

echo "#############################"

echo "Initialization complete!"

echo "INDEX_FACTORY_ADDR=$INDEX_FACTORY_ADDR"
echo "SWAP_UTILITY_ADDR=$SWAP_UTILITY_ADDR"
echo "INDEX_WASM_HASH=$INDEX_WASM_HASH"
