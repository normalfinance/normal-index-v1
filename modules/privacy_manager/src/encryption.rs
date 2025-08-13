use soroban_sdk::{Address, Bytes, Env};

/// Encrypt component weight for authorized viewers
/// 
/// This uses a simple XOR-based encryption scheme suitable for Soroban's constraints.
/// In production, this could be enhanced with proper AES encryption if available.
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `weight` - Component weight to encrypt
/// * `encryption_key` - Encryption key derived from viewer authorization
/// 
/// # Returns
/// * Encrypted weight as Bytes
pub fn encrypt_component_weight(e: &Env, weight: u128, encryption_key: &Bytes) -> Bytes {
    let weight_bytes = weight.to_be_bytes();
    let mut encrypted = Bytes::new(e);
    
    // Simple XOR encryption (can be enhanced to AES if Soroban supports it)
    for (i, byte) in weight_bytes.iter().enumerate() {
        let key_idx = (i as u32) % encryption_key.len();
        let key_byte = encryption_key.get(key_idx).unwrap_or(0);
        let encrypted_byte = byte ^ key_byte;
        encrypted.push_back(encrypted_byte);
    }
    
    encrypted
}

/// Decrypt component weight for authorized viewers
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `encrypted_weight` - Encrypted weight bytes
/// * `encryption_key` - Decryption key
/// 
/// # Returns
/// * Decrypted weight as u128, or None if decryption fails
/// encryption_key -> private key of the admin + (some timestamp +- 5 seconds)
pub fn decrypt_component_weight(_e: &Env, encrypted_weight: &Bytes, encryption_key: &Bytes) -> Option<u128> {
    // admin.require_auth(); will be needed here
    if encrypted_weight.len() != 16 {
        // u128 should be exactly 16 bytes
        return None;
    }

    // check if the encryption key is valid
    //let decrypted_key = derivation(encryption_key)
    let mut decrypted_bytes = [0u8; 16];
    
    for (i, encrypted_byte) in encrypted_weight.iter().enumerate() {
        let key_idx = (i as u32) % encryption_key.len();
        let key_byte = encryption_key.get(key_idx).unwrap_or(0);
        decrypted_bytes[i] = encrypted_byte ^ key_byte;
    }
    
    Some(u128::from_be_bytes(decrypted_bytes))
}

/// Generate encryption key for authorized viewer
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `viewer` - Address of authorized viewer
/// * `master_key` - Master encryption key for the index
/// 
/// # Returns
/// * Derived encryption key for the viewer
pub fn derive_viewer_encryption_key(e: &Env, viewer: &Address, master_key: &Bytes) -> Bytes {
    let mut key_data = Bytes::new(e);
    
    // Add viewer address as simple bytes representation
    let viewer_string = viewer.to_string();
    // Use the string length and first few characters as entropy
    let len_bytes = (viewer_string.len() as u32).to_be_bytes();
    for byte in len_bytes.iter() {
        key_data.push_back(*byte);
    }
    
    // Add master key
    for byte in master_key.iter() {
        key_data.push_back(byte);
    }
    
    // Hash to create derived key
    e.crypto().sha256(&key_data).into()
}

/// Verify encryption key hash
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `key` - Encryption key to verify
/// * `expected_hash` - Expected hash of the key
/// 
/// # Returns
/// * true if key hash matches expected hash
pub fn verify_encryption_key_hash(e: &Env, key: &Bytes, expected_hash: &Bytes) -> bool {
    let computed_hash: Bytes = e.crypto().sha256(key).into();
    computed_hash == *expected_hash
}

/// Encrypt multiple component weights for batch operations
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `weights` - Vector of weights to encrypt
/// * `encryption_key` - Encryption key
/// 
/// # Returns
/// * Vector of encrypted weights
pub fn encrypt_component_weights_batch(
    e: &Env,
    weights: &[u128],
    encryption_key: &Bytes,
) -> soroban_sdk::Vec<Bytes> {
    let mut encrypted_weights = soroban_sdk::Vec::new(e);
    
    for weight in weights.iter() {
        let encrypted = encrypt_component_weight(e, *weight, encryption_key);
        encrypted_weights.push_back(encrypted);
    }
    
    encrypted_weights
}

/// Decrypt multiple component weights for batch operations
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `encrypted_weights` - Vector of encrypted weights
/// * `encryption_key` - Decryption key
/// 
/// # Returns
/// * Vector of decrypted weights, None if any decryption fails
pub fn decrypt_component_weights_batch(
    e: &Env,
    encrypted_weights: &soroban_sdk::Vec<Bytes>,
    encryption_key: &Bytes,
) -> Option<soroban_sdk::Vec<u128>> {
    let mut decrypted_weights = soroban_sdk::Vec::new(e);
    
    for encrypted_weight in encrypted_weights.iter() {
        match decrypt_component_weight(e, &encrypted_weight, encryption_key) {
            Some(weight) => decrypted_weights.push_back(weight),
            None => return None, // Fail if any decryption fails
        }
    }
    
    Some(decrypted_weights)
}

/// Create master encryption key from seed
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `seed` - Seed value for key generation
/// * `admin_address` - Admin address for additional entropy
/// 
/// # Returns
/// * Master encryption key
pub fn create_master_encryption_key(e: &Env, seed: u64, admin_address: &Address) -> Bytes {
    let mut key_data = Bytes::new(e);
    
    // Add seed
    let seed_bytes = seed.to_be_bytes();
    for byte in seed_bytes.iter() {
        key_data.push_back(*byte);
    }
    
    // Add admin address for additional entropy
    let admin_string = admin_address.to_string();
    // Use the string length and first few characters as entropy
    let admin_len_bytes = (admin_string.len() as u32).to_be_bytes();
    for byte in admin_len_bytes.iter() {
        key_data.push_back(*byte);
    }
    
    // Add current timestamp for uniqueness
    let timestamp = e.ledger().timestamp();
    let timestamp_bytes = timestamp.to_be_bytes();
    for byte in timestamp_bytes.iter() {
        key_data.push_back(*byte);
    }
    
    // Hash to create master key
    e.crypto().sha256(&key_data).into()
}

/// Verify authorized viewer access
/// 
/// # Arguments
/// * `viewer` - Address requesting access
/// * `authorized_viewers` - List of authorized viewer addresses
/// 
/// # Returns
/// * true if viewer is authorized
pub fn verify_viewer_authorization(
    viewer: &Address,
    authorized_viewers: &soroban_sdk::Vec<Address>,
) -> bool {
    for authorized_viewer in authorized_viewers.iter() {
        if viewer == &authorized_viewer {
            return true;
        }
    }
    false
}