use soroban_sdk::{Address, Env};
use crate::types::{ViewerAccessLevel, IndexPrivacyConfig};

/// Determine access level for a viewer based on privacy configuration
/// 
/// # Arguments
/// * `viewer` - Address requesting access
/// * `privacy_config` - Privacy configuration for the index
/// 
/// # Returns
/// * ViewerAccessLevel for the given viewer
pub fn get_viewer_access_level(
    viewer: &Address,
    privacy_config: &IndexPrivacyConfig,
) -> ViewerAccessLevel {
    // Check emergency access first (highest priority)
    for emergency_address in privacy_config.emergency_access.iter() {
        if viewer == &emergency_address {
            return ViewerAccessLevel::Emergency;
        }
    }
    
    // Check compliance authorities
    for compliance_address in privacy_config.compliance_authorities.iter() {
        if viewer == &compliance_address {
            return ViewerAccessLevel::Compliance;
        }
    }
    
    // Check authorized viewers
    for authorized_address in privacy_config.authorized_viewers.iter() {
        if viewer == &authorized_address {
            return ViewerAccessLevel::Authorized;
        }
    }
    
    // Default to public access
    ViewerAccessLevel::Public
}


/// Check if viewer can access component weights
/// 
/// # Arguments
/// * `viewer` - Address requesting access
/// * `privacy_config` - Privacy configuration for the index
/// 
/// # Returns
/// * true if viewer can access component weights
pub fn can_view_component_weights(
    viewer: &Address,
    privacy_config: &IndexPrivacyConfig,
) -> bool {
    let access_level = get_viewer_access_level(viewer, privacy_config);
    
    matches!(
        access_level,
        ViewerAccessLevel::Authorized
            | ViewerAccessLevel::Compliance
            | ViewerAccessLevel::Emergency
    )
}

/// Check if viewer can view detailed portfolio breakdown
/// 
/// # Arguments
/// * `viewer` - Address requesting access
/// * `privacy_config` - Privacy configuration for the index
/// 
/// # Returns
/// * true if viewer can access detailed portfolio information
pub fn can_view_detailed_portfolio(
    viewer: &Address,
    privacy_config: &IndexPrivacyConfig,
) -> bool {
    let access_level = get_viewer_access_level(viewer, privacy_config);
    
    matches!(
        access_level,
        ViewerAccessLevel::Investor
            | ViewerAccessLevel::Authorized
            | ViewerAccessLevel::Compliance
            | ViewerAccessLevel::Emergency
    )
}

/// Check if viewer can modify privacy settings
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `viewer` - Address requesting access
/// * `privacy_config` - Privacy configuration for the index
/// 
/// # Returns
/// * true if viewer can modify privacy settings
pub fn can_modify_privacy_settings(
    _e: &Env,
    viewer: &Address,
    privacy_config: &IndexPrivacyConfig,
) -> bool {
    // Only emergency access and admin can modify privacy settings
    let access_level = get_viewer_access_level(viewer, privacy_config);
    
    match access_level {
        ViewerAccessLevel::Emergency => true,
        _ => {
            // Check if viewer is admin using access control
            // This would need to be implemented with the existing access control system
            false // Placeholder - should check admin role
        }
    }
}

/// Validate viewer authorization for specific operations
/// 
/// # Arguments
/// * `viewer` - Address requesting access
/// * `required_level` - Minimum access level required
/// * `privacy_config` - Privacy configuration for the index
/// 
/// # Returns
/// * true if viewer has sufficient access level
pub fn validate_viewer_authorization(
    viewer: &Address,
    required_level: ViewerAccessLevel,
    privacy_config: &IndexPrivacyConfig,
) -> bool {
    let viewer_level = get_viewer_access_level(viewer, privacy_config);
    
    match required_level {
        ViewerAccessLevel::Public => true, // Everyone has public access
        ViewerAccessLevel::Investor => matches!(
            viewer_level,
            ViewerAccessLevel::Investor
                | ViewerAccessLevel::Authorized
                | ViewerAccessLevel::Compliance
                | ViewerAccessLevel::Emergency
        ),
        ViewerAccessLevel::Authorized => matches!(
            viewer_level,
            ViewerAccessLevel::Authorized
                | ViewerAccessLevel::Compliance
                | ViewerAccessLevel::Emergency
        ),
        ViewerAccessLevel::Compliance => matches!(
            viewer_level,
            ViewerAccessLevel::Compliance | ViewerAccessLevel::Emergency
        ),
        ViewerAccessLevel::Emergency => matches!(viewer_level, ViewerAccessLevel::Emergency),
    }
}

/// Get maximum allowed detail level for viewer
/// 
/// # Arguments
/// * `viewer` - Address requesting access
/// * `privacy_config` - Privacy configuration for the index
/// 
/// # Returns
/// * Maximum detail level the viewer can access
pub fn get_max_detail_level(
    viewer: &Address,
    privacy_config: &IndexPrivacyConfig,
) -> ViewerAccessLevel {
    get_viewer_access_level(viewer, privacy_config)
}

/// Check if address is in authorized list
/// 
/// # Arguments
/// * `address` - Address to check
/// * `authorized_list` - List of authorized addresses
/// 
/// # Returns
/// * true if address is in the list
pub fn is_address_authorized(
    address: &Address,
    authorized_list: &soroban_sdk::Vec<Address>,
) -> bool {
    for authorized in authorized_list.iter() {
        if address == &authorized {
            return true;
        }
    }
    false
}

/// Validate privacy configuration
/// 
/// # Arguments
/// * `privacy_config` - Privacy configuration to validate
/// 
/// # Returns
/// * true if configuration is valid
pub fn validate_privacy_config(privacy_config: &IndexPrivacyConfig) -> bool {
    // Ensure no duplicate addresses across different access levels
    let mut all_addresses = soroban_sdk::Vec::new(&Env::default());
    
    // Add all addresses to check for duplicates
    for addr in privacy_config.authorized_viewers.iter() {
        all_addresses.push_back(addr);
    }
    
    for addr in privacy_config.compliance_authorities.iter() {
        all_addresses.push_back(addr);
    }
    
    for addr in privacy_config.emergency_access.iter() {
        all_addresses.push_back(addr);
    }
    
    // Additional validation logic can be added here
    // For now, just return true
    true
}

/// Create default privacy configuration for public index
/// 
/// # Arguments
/// * `e` - Soroban environment
/// 
/// # Returns
/// * Default public privacy configuration
pub fn create_public_privacy_config(e: &Env) -> IndexPrivacyConfig {
    IndexPrivacyConfig {
        privacy_mode: crate::types::IndexPrivacyMode::Public,
        authorized_viewers: soroban_sdk::Vec::new(e),
        compliance_authorities: soroban_sdk::Vec::new(e),
        emergency_access: soroban_sdk::Vec::new(e),
        encryption_key_hash: None,
    }
}

/// Create privacy configuration for private index
/// 
/// # Arguments
/// * `e` - Soroban environment
/// * `admin` - Admin address (gets emergency access)
/// * `authorized_viewers` - Initial authorized viewers
/// * `compliance_authorities` - Compliance authority addresses
/// 
/// # Returns
/// * Privacy configuration for private index
pub fn create_private_privacy_config(
    e: &Env,
    admin: &Address,
    authorized_viewers: soroban_sdk::Vec<Address>,
    compliance_authorities: soroban_sdk::Vec<Address>,
) -> IndexPrivacyConfig {
    let mut emergency_access = soroban_sdk::Vec::new(e);
    emergency_access.push_back(admin.clone());
    
    IndexPrivacyConfig {
        privacy_mode: crate::types::IndexPrivacyMode::Private,
        authorized_viewers,
        compliance_authorities,
        emergency_access,
        encryption_key_hash: None, // Will be set when encryption key is created
    }
}