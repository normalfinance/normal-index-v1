use soroban_sdk::{Address, Bytes, Env};

/// Create a hash commitment for a component weight
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `weight` - Component weight in basis points (0-10000)
/// * `salt` - Random salt for security
/// 
/// # Returns
/// * Hash commitment as Bytes
pub fn create_weight_commitment(e: &Env, weight: u128, salt: u64) -> Bytes {
    let mut data = Bytes::new(e);
    
    // Add weight as bytes
    let weight_bytes = weight.to_be_bytes();
    for byte in weight_bytes.iter() {
        data.push_back(*byte);
    }
    
    // Add salt as bytes
    let salt_bytes = salt.to_be_bytes();
    for byte in salt_bytes.iter() {
        data.push_back(*byte);
    }
    
    // Create SHA-256 hash commitment
    e.crypto().sha256(&data).into()
}

//instead of storing weights, we can store commitments (hash) to the weights

/// Verify a weight commitment
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `commitment` - Hash commitment to verify
/// * `weight` - Claimed weight
/// * `salt` - Salt used in commitment
/// 
/// # Returns
/// * true if commitment is valid, false otherwise
pub fn verify_weight_commitment(e: &Env, commitment: &Bytes, weight: u128, salt: u64) -> bool {
    let computed_commitment = create_weight_commitment(e, weight, salt);
    commitment == &computed_commitment
}

/// Create commitments for multiple component weights
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `components` - Vector of (asset_address, weight) pairs
/// * `master_salt` - Master salt for all commitments

pub fn commit_component_weights(
    e: &Env,
    components: &[(Address, u128)],
    master_salt: u64,
) -> bool {
    
    for (i, (_asset_address, weight)) in components.iter().enumerate() {
        // Use master salt + index for each component to ensure uniqueness
        let component_salt = master_salt.wrapping_add(i as u64);
        
        let commitment = create_weight_commitment(e, *weight, component_salt);   

        e.storage().set(component_salt, &commitment);
    }
    
    true
}

/// Verify that a set of component weights sum to exactly 10000 basis points (100%)
/// 
/// # Arguments
/// * `weights` - Vector of component weights
/// 
/// # Returns
/// * true if weights sum to 10000, false otherwise
pub fn verify_weight_sum(weights: &[u128]) -> bool {
    let total: u128 = weights.iter().sum();
    total == 10000 // 10000 basis points = 100%
}


/// Verify individual component weight is within valid range
/// 
/// # Arguments
/// * `weight` - Component weight in basis points
/// * `max_weight` - Maximum allowed weight for single component
/// 
/// # Returns
/// * true if weight is valid, false otherwise
pub fn verify_weight_bounds(weight: u128, max_weight: u128) -> bool {
    weight <= max_weight && weight > 0
}

/// Verify rebalancing constraints
/// 
/// # Arguments
/// * `old_weights` - Current weights
/// * `new_weights` - Proposed new weights
/// * `max_individual_weight` - Maximum weight for any single component
/// * `max_change_per_component` - Maximum change allowed per component
/// 
/// # Returns
/// * true if rebalancing is within constraints
pub fn verify_rebalancing_constraints(
    old_weights: &[u128],
    new_weights: &[u128],
    max_individual_weight: u128,
    max_change_per_component: u128,
) -> bool {
    // Check that arrays have same length
    if old_weights.len() != new_weights.len() {
        return false;
    }
    
    // Verify new weights sum to 100%
    if !verify_weight_sum(new_weights) {
        return false;
    }
    
    // Check individual constraints
    for (old_weight, new_weight) in old_weights.iter().zip(new_weights.iter()) {
        // Check individual weight bounds
        if !verify_weight_bounds(*new_weight, max_individual_weight) {
            return false;
        }
        
        // Check change limits
        let weight_change = if new_weight > old_weight {
            new_weight - old_weight
        } else {
            old_weight - new_weight
        };
        
        if weight_change > max_change_per_component {
            return false;
        }
    }
    
    true
}