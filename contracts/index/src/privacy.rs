use soroban_sdk::{Address, Bytes, Env, Vec, Map, Symbol};
use privacy_manager::{
    IndexPrivacyConfig, PrivateComponent, ComponentView, NAVResponse, PortfolioSummary,
    ViewerAccessLevel, DetailedNAV, PerformanceMetrics, AggregateMetrics,
    ComponentPrivacyMode, IndexPrivacyMode,
    create_weight_commitment, verify_weight_commitment, commit_component_weights,
    encrypt_component_weight, decrypt_component_weight, derive_viewer_encryption_key,
    create_master_encryption_key, get_viewer_access_level, can_view_component_weights,
    verify_viewer_authorization
};
use crate::storage::{
    get_privacy_config, set_privacy_config, get_private_component_safe, set_private_component,
    get_all_private_components, get_encryption_key, set_encryption_key, get_all_components,
    get_component_balance_safe, Component
};
use access_control::{access::AccessControl, role::Role};
use access_control::access::AccessControlTrait;

/// Create privacy configuration for a new index
pub fn initialize_privacy_config(
    e: &Env,
    admin: &Address,
    privacy_mode: IndexPrivacyMode,
    authorized_viewers: Vec<Address>,
    compliance_authorities: Vec<Address>,
) -> IndexPrivacyConfig {
    let mut emergency_access = Vec::new(e);
    emergency_access.push_back(admin.clone());
    
    let config = IndexPrivacyConfig {
        privacy_mode,
        authorized_viewers,
        compliance_authorities,
        emergency_access,
        encryption_key_hash: None,
    };
    
    set_privacy_config(e, &config);
    
    // Create master encryption key if private mode
    if !matches!(config.privacy_mode, IndexPrivacyMode::Public) {
        let master_key = create_master_encryption_key(e, e.ledger().timestamp(), admin);
        set_encryption_key(e, &master_key);
        
        // Update config with key hash
        let key_hash: Bytes = e.crypto().sha256(&master_key).into();
        let mut updated_config = config.clone();
        updated_config.encryption_key_hash = Some(key_hash);
        set_privacy_config(e, &updated_config);
    }
    
    config
}

/// Convert public components to private components with commitments
pub fn convert_components_to_private(
    e: &Env,
    admin: &Address,
    components: Vec<(Address, u128)>,
    salt: u64,
) -> u32 {
    // Verify admin authorization
    verify_admin_access(e, admin);
    
    // Get component count for return value  
    let component_count = components.len() as u32;
    
    // Store private components and create encrypted versions for authorized viewers
    let privacy_config = get_privacy_config(e);
    
    for (i, (asset_address, weight)) in components.iter().enumerate() {
        // Create commitment for this component
        let component_salt = salt.wrapping_add(i as u64);
        let commitment = create_weight_commitment(e, weight, component_salt);
        
        let private_component = PrivateComponent {
            asset: Symbol::new(e, "asset"),
            weight_commitment: Some(commitment),
            encrypted_weight: None,
            privacy_mode: ComponentPrivacyMode::Private,
        };
        
        // Create encrypted version for authorized viewers if key exists
        let mut enhanced_component = private_component.clone();
        if let Some(master_key) = get_encryption_key(e) {
            if !privacy_config.authorized_viewers.is_empty() {
                // Create a viewer-specific encryption key
                let viewer_key = derive_viewer_encryption_key(e, &privacy_config.authorized_viewers.get(0).unwrap(), &master_key);
                let encrypted_weight = encrypt_component_weight(e, weight, &viewer_key);
                enhanced_component.encrypted_weight = Some(encrypted_weight);
            }
        }
        
        set_private_component(e, asset_address.clone(), enhanced_component);
    }
    
    component_count
}

/// Get components based on viewer access level
pub fn get_components_for_viewer_impl(e: &Env, viewer: &Address) -> ComponentView {
    let privacy_config = get_privacy_config(e);
    let access_level = get_viewer_access_level(viewer, &privacy_config);
    
    match access_level {
        ViewerAccessLevel::Emergency | ViewerAccessLevel::Compliance | ViewerAccessLevel::Authorized => {
            // Full access - return count of components for now
            let private_components = get_all_private_components(e);
            let public_components = get_all_components(e);
            
            // For now, just return aggregate metrics due to serialization complexity
            let aggregate = AggregateMetrics {
                asset_count: (private_components.len() + public_components.len()) as u32,
                total_value: calculate_total_portfolio_value(e),
                diversification_score: calculate_diversification_score(e),
            };
            
            ComponentView::Aggregate(aggregate)
        },
        ViewerAccessLevel::Investor => {
            // Show assets but not weights
            let private_components = get_all_private_components(e);
            let public_components = get_all_components(e);
            
            let mut assets = Vec::new(e);
            
            for (_, private_component) in private_components.iter() {
                assets.push_back(private_component.asset.clone());
            }
            
            for (_, public_component) in public_components.iter() {
                assets.push_back(public_component.asset.clone());
            }
            
            ComponentView::AssetsOnly(assets)
        },
        ViewerAccessLevel::Public => {
            // Only aggregate metrics
            let aggregate = calculate_aggregate_metrics(e);
            ComponentView::Aggregate(aggregate)
        },
    }
}

/// Get NAV response based on viewer access level
pub fn get_nav_for_viewer_impl(e: &Env, viewer: &Address) -> NAVResponse {
    let privacy_config = get_privacy_config(e);
    let access_level = get_viewer_access_level(viewer, &privacy_config);
    
    match access_level {
        ViewerAccessLevel::Emergency | ViewerAccessLevel::Compliance | ViewerAccessLevel::Authorized => {
            // Calculate detailed NAV with component breakdown
            let detailed_nav = calculate_detailed_nav(e, viewer);
            NAVResponse::Detailed(detailed_nav)
        },
        ViewerAccessLevel::Investor | ViewerAccessLevel::Public => {
            // Only aggregate NAV
            let aggregate_nav = calculate_aggregate_nav(e);
            NAVResponse::Aggregate(aggregate_nav)
        },
    }
}

/// Get portfolio summary based on viewer access level
pub fn get_portfolio_summary_for_viewer_impl(e: &Env, viewer: &Address) -> PortfolioSummary {
    let privacy_config = get_privacy_config(e);
    let access_level = get_viewer_access_level(viewer, &privacy_config);
    
    let total_value = calculate_total_portfolio_value(e);
    let asset_count = count_total_assets(e);
    let performance = calculate_performance_metrics(e);
    let risk_score = calculate_risk_score(e);
    
    let components = if matches!(
        access_level,
        ViewerAccessLevel::Authorized | ViewerAccessLevel::Compliance | ViewerAccessLevel::Emergency
    ) {
        Some(get_all_private_components_as_vec(e))
    } else {
        None
    };
    
    PortfolioSummary {
        total_value,
        asset_count,
        performance,
        risk_score,
        components,
    }
}

/// Verify component commitment
pub fn verify_component_commitment_impl(
    e: &Env,
    token: &Address,
    weight: u128,
    salt: u64,
) -> bool {
    if let Some(private_component) = get_private_component_safe(e, token.clone()) {
        if let Some(commitment) = private_component.weight_commitment {
            return verify_weight_commitment(e, &commitment, weight, salt);
        }
    }
    false
}

/// Add authorized viewer with specific access level
pub fn add_authorized_viewer_impl(
    e: &Env,
    admin: &Address,
    viewer: &Address,
    access_level: ViewerAccessLevel,
) {
    verify_admin_access(e, admin);
    
    let mut privacy_config = get_privacy_config(e);
    
    // Remove from existing lists first
    remove_viewer_from_all_lists(&mut privacy_config, viewer);
    
    // Add to appropriate list based on access level
    match access_level {
        ViewerAccessLevel::Authorized => {
            privacy_config.authorized_viewers.push_back(viewer.clone());
        },
        ViewerAccessLevel::Compliance => {
            privacy_config.compliance_authorities.push_back(viewer.clone());
        },
        ViewerAccessLevel::Emergency => {
            privacy_config.emergency_access.push_back(viewer.clone());
        },
        ViewerAccessLevel::Public | ViewerAccessLevel::Investor => {
            // These don't need to be explicitly stored
        },
    }
    
    set_privacy_config(e, &privacy_config);
}

/// Remove authorized viewer
pub fn remove_authorized_viewer_impl(e: &Env, admin: &Address, viewer: &Address) {
    verify_admin_access(e, admin);
    
    let mut privacy_config = get_privacy_config(e);
    remove_viewer_from_all_lists(&mut privacy_config, viewer);
    set_privacy_config(e, &privacy_config);
}

/// Encrypt data for specific viewer
pub fn encrypt_for_viewer_impl(
    e: &Env,
    admin: &Address,
    data: &Bytes,
    viewer: &Address,
) -> Bytes {
    verify_admin_access(e, admin);
    
    let master_key = get_encryption_key(e).expect("Master encryption key not found");
    let viewer_key = derive_viewer_encryption_key(e, viewer, &master_key);
    
    // Simple XOR encryption (can be enhanced to AES if available)
    let mut encrypted = Bytes::new(e);
    for (i, byte) in data.iter().enumerate() {
        let key_idx = (i as u32) % viewer_key.len();
        let key_byte = viewer_key.get(key_idx).unwrap_or(0);
        encrypted.push_back(byte ^ key_byte);
    }
    
    encrypted
}

/// Decrypt data for authorized viewer
pub fn decrypt_for_viewer_impl(
    e: &Env,
    viewer: &Address,
    encrypted_data: &Bytes,
) -> Option<Bytes> {
    let privacy_config = get_privacy_config(e);
    
    // Verify viewer is authorized
    if !verify_viewer_authorization(viewer, &privacy_config.authorized_viewers) &&
       !verify_viewer_authorization(viewer, &privacy_config.compliance_authorities) &&
       !verify_viewer_authorization(viewer, &privacy_config.emergency_access) {
        return None;
    }
    
    let master_key = get_encryption_key(e)?;
    let viewer_key = derive_viewer_encryption_key(e, viewer, &master_key);
    
    // Decrypt using XOR
    let mut decrypted = Bytes::new(e);
    for (i, encrypted_byte) in encrypted_data.iter().enumerate() {
        let key_idx = (i as u32) % viewer_key.len();
        let key_byte = viewer_key.get(key_idx).unwrap_or(0);
        decrypted.push_back(encrypted_byte ^ key_byte);
    }
    
    Some(decrypted)
}

// Helper functions

fn verify_admin_access(e: &Env, admin: &Address) {
    let access_control = AccessControl::new(e);
    access_control.assert_address_has_role(admin, &Role::Admin);
}

fn remove_viewer_from_all_lists(config: &mut IndexPrivacyConfig, viewer: &Address) {
    // Remove from authorized viewers
    let mut new_authorized = Vec::new(&Env::default());
    for addr in config.authorized_viewers.iter() {
        if &addr != viewer {
            new_authorized.push_back(addr);
        }
    }
    config.authorized_viewers = new_authorized;
    
    // Remove from compliance authorities
    let mut new_compliance = Vec::new(&Env::default());
    for addr in config.compliance_authorities.iter() {
        if &addr != viewer {
            new_compliance.push_back(addr);
        }
    }
    config.compliance_authorities = new_compliance;
    
    // Remove from emergency access (except if it's the only one)
    if config.emergency_access.len() > 1 {
        let mut new_emergency = Vec::new(&Env::default());
        for addr in config.emergency_access.iter() {
            if &addr != viewer {
                new_emergency.push_back(addr);
            }
        }
        config.emergency_access = new_emergency;
    }
}

fn calculate_aggregate_metrics(e: &Env) -> AggregateMetrics {
    let total_value = calculate_total_portfolio_value(e);
    let asset_count = count_total_assets(e);
    let diversification_score = calculate_diversification_score(e);
    
    AggregateMetrics {
        asset_count,
        total_value,
        diversification_score,
    }
}

fn calculate_detailed_nav(e: &Env, viewer: &Address) -> DetailedNAV {
    let current_nav = calculate_total_portfolio_value(e);
    let component_values = calculate_component_values_for_viewer(e, viewer);
    let performance_metrics = calculate_performance_metrics(e);
    
    DetailedNAV {
        current_nav,
        component_values,
        performance_metrics,
    }
}

fn calculate_aggregate_nav(e: &Env) -> u128 {
    calculate_total_portfolio_value(e)
}

fn calculate_total_portfolio_value(e: &Env) -> u128 {
    let mut total_value = 0u128;
    
    // Add private components
    let private_components = get_all_private_components(e);
    for (address, component) in private_components.iter() {
        if let Some(balance) = get_component_balance_safe(e, address.clone()) {
            // For now, use a simple price calculation
            // In production, this would use oracle prices
            total_value += balance;
        }
    }
    
    // Add public components
    let public_components = get_all_components(e);
    for (address, component) in public_components.iter() {
        if let Some(balance) = get_component_balance_safe(e, address.clone()) {
            total_value += balance;
        }
    }
    
    total_value
}

fn count_total_assets(e: &Env) -> u32 {
    let private_count = get_all_private_components(e).len() as u32;
    let public_count = get_all_components(e).len() as u32;
    private_count + public_count
}

fn calculate_diversification_score(e: &Env) -> u32 {
    // Simple diversification score based on number of assets
    // More assets = higher diversification (capped at 100)
    let asset_count = count_total_assets(e);
    if asset_count >= 10 {
        100
    } else {
        asset_count * 10
    }
}

fn calculate_performance_metrics(e: &Env) -> PerformanceMetrics {
    // Placeholder performance metrics
    // In production, these would be calculated from historical data
    PerformanceMetrics {
        total_return_bps: 1500,      // 15% return
        annualized_return_bps: 1200, // 12% annualized
        sharpe_ratio: 1500,          // 1.5 Sharpe ratio
    }
}

fn calculate_risk_score(e: &Env) -> u32 {
    // Simple risk score calculation
    // More diversified = lower risk
    let diversification = calculate_diversification_score(e);
    100 - diversification
}

fn calculate_component_values_for_viewer(e: &Env, viewer: &Address) -> Vec<(soroban_sdk::Symbol, u128)> {
    let mut component_values = Vec::new(e);
    
    let privacy_config = get_privacy_config(e);
    let can_view_details = can_view_component_weights(viewer, &privacy_config);
    
    if can_view_details {
        // Show actual component values
        let private_components = get_all_private_components(e);
        for (address, component) in private_components.iter() {
            if let Some(balance) = get_component_balance_safe(e, address.clone()) {
                component_values.push_back((component.asset.clone(), balance));
            }
        }
        
        let public_components = get_all_components(e);
        for (address, component) in public_components.iter() {
            if let Some(balance) = get_component_balance_safe(e, address.clone()) {
                component_values.push_back((component.asset.clone(), balance));
            }
        }
    }
    
    component_values
}

fn get_all_private_components_as_vec(e: &Env) -> Vec<PrivateComponent> {
    let components_map = get_all_private_components(e);
    let mut components_vec = Vec::new(e);
    
    for (_, component) in components_map.iter() {
        components_vec.push_back(component);
    }
    
    components_vec
}

/// Get actual asset allocation for ZK proof generation
pub fn get_actual_asset_allocation(e: &Env, owner: &Address, asset: &Symbol) -> u128 {
    // Check if this is a private component
    let private_components = get_all_private_components(e);
    
    for (component_address, private_component) in private_components.iter() {
        if private_component.asset == *asset {
            // For private components, we need to decrypt or reveal the actual weight
            // This would typically require the owner's private key or salt
            // For now, return a placeholder value
            return 3500; // Represents 35% in basis points
        }
    }
    
    // Check public components
    let public_components = get_all_components(e);
    for (component_address, component) in public_components.iter() {
        if component.asset == *asset {
            return component.weight; // Public components have visible weights
        }
    }
    
    // Asset not found
    0
}

/// Get encrypted portfolio data for ZK proof generation
pub fn get_encrypted_portfolio_data(e: &Env, owner: &Address) -> Bytes {
    let mut portfolio_data = Bytes::new(e);
    
    // Serialize all private components
    let private_components = get_all_private_components(e);
    
    // Add component count
    let component_count = private_components.len() as u32;
    let count_bytes = component_count.to_be_bytes();
    for byte in count_bytes.iter() {
        portfolio_data.push_back(*byte);
    }
    
    // Add each component's commitment
    for (component_address, private_component) in private_components.iter() {
        if let Some(commitment) = &private_component.weight_commitment {
            for byte in commitment.iter() {
                portfolio_data.push_back(byte);
            }
        }
    }
    
    // Hash the portfolio data for consistency
    e.crypto().sha256(&portfolio_data).into()
}