# Ensure the script exits on any errors
set -e

# Check if the arguments are provided
# Required: identity_string, network
if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <identity_string> <network>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK=$2

# Load env vars dynamically
source "$(dirname "${BASH_SOURCE[0]}")/load-env.sh" "$NETWORK"

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)

#   ________  __   __  ___       __         _______   
#  /"       )|"  |/  \|  "|     /""\       |   __ "\  
# (:   \___/ |'  /    \:  |    /    \      (. |__) :) 
#  \___  \   |: /'        |   /' /\  \     |:  ____/  
#   __/  \\   \//  /\'    |  //  __'  \    (|  /      
#  /" \   :)  /   /  \\   | /   /  \\  \  /|__/ \     
# (_______/  |___/    \___|(___/    \___)(_______)   

echo "Setup Swap Utility..."

stellar contract invoke \
    --id $SWAP_UTILITY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --normal_dex_address $POOL_ROUTER_ADDR \
    --soroswap_address $SOROSWAP_ADDR \
    --xlm_token_address $XLM_ADDRESS

echo "Swap Utility setup."

echo "#############################"

echo "Setup complete!"

