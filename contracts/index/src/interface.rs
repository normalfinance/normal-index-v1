use soroban_sdk::{contracttype, Address, Env, Map, Vec, Bytes};

use crate::stake::Stake;
use crate::storage::Component;
use privacy_manager::{
    IndexPrivacyConfig, PrivateComponent, ComponentView, NAVResponse, PortfolioSummary,
    ViewerAccessLevel, DetailedNAV, PerformanceMetrics, AggregateMetrics, CommitmentProof,
    RangeProof, SelectiveRevelationRequest, SelectiveRevelationResult,
    ComplianceConstraint, ProofMethod
};

pub trait IndexTrait {
    fn mint(
        e: Env,
        user: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
        max_slippage: Option<u64>,
    );

    fn redeem(e: Env, user: Address, share_amount: u128);

    fn get_token(e: Env) -> Address;

    fn get_factory(e: Env) -> Address;

    fn get_base_nav(e: Env) -> u128;

    fn get_initial_price(e: Env) -> u128;

    fn get_nav(e: Env) -> i128;

    fn get_price(e: Env) -> i128;

    fn get_total_shares(e: Env) -> u128;

    fn get_public_status(e: Env) -> bool;

    fn get_whitelist_status(e: Env, address: Address) -> bool;

    fn get_blacklist_status(e: Env, address: Address) -> bool;

    fn get_manager_fee_fraction(e: Env) -> u32;

    fn get_rebalance_threshold(e: Env) -> u64;

    fn get_last_rebalance_timestamp(e: Env) -> u64;

    fn get_last_updated_timestamp(e: Env) -> u64;

    fn get_total_mints(e: Env) -> u128;

    fn get_total_redemptions(e: Env) -> u128;

    fn get_total_fees(e: Env) -> u128;

    fn get_component(e: Env, token: Address) -> crate::storage::Component;

    fn get_component_balance(e: Env, token: Address) -> u128;

    fn get_last_fee_collection(e: Env) -> u64;

    /// Transfer shares between users with proper fee handling
    fn transfer_shares(e: Env, from: Address, to: Address, amount: u128);

    /// Transfer shares from allowance with proper fee handling  
    fn transfer_shares_from(e: Env, spender: Address, from: Address, to: Address, amount: u128);
}

pub trait AdminInterface {
    fn initialize(e: Env, admin: Address, token: Address);

    fn rebalance(e: Env, caller: Address, params: RebalanceParams);

    fn set_rebalance_authority(e: Env, admin: Address, authority: Address, status: bool);

    fn distribute_manager_fees(e: Env, admin: Address);

    fn distribute_protocol_fees(e: Env, admin: Address);

    fn set_factory(e: Env, admin: Address, factory: Address);

    fn set_base_nav(e: Env, admin: Address, base_nav: u128);

    fn set_initial_price(e: Env, admin: Address, initial_price: u128);

    fn set_public_status(e: Env, admin: Address, public: bool);

    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_manager_address(e: Env, admin: Address, manager: Address);

    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address);

    fn set_manager_fee_fraction(e: Env, admin: Address, fee_fraction: u32);

    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64);

    fn set_unstaking_period(e: Env, admin: Address, unstaking_period: u64);

    fn set_max_shares(e: Env, admin: Address, max_shares: u128);

    //    _______     __       ____  ____   ________  _______  ________
    //   |   __ "\   /""\     ("  _||_ " | /"       )/"     "||"      "\
    //   (. |__) :) /    \    |   (  ) : |(:   \___/(: ______)(.  ___  :)
    //   |:  ____/ /' /\  \   (:  |  | . ) \___  \   \/    |  |: \   ) ||
    //   (|  /    //  __'  \   \\ \__/ //   __/  \\  // ___)_ (| (___\ ||
    //  /|__/ \  /   /  \\  \  /\\ __ //\  /" \   :)(:      "||:       :)
    // (_______)(___/    \___)(__________)(_______/  \_______)(________/

    fn kill_mint(e: Env, admin: Address);
    fn kill_redeem(e: Env, admin: Address);
    fn kill_rebalance(e: Env, admin: Address);

    fn unkill_mint(e: Env, admin: Address);
    fn unkill_redeem(e: Env, admin: Address);
    fn unkill_rebalance(e: Env, admin: Address);

    fn get_is_killed_mint(e: Env) -> bool;
    fn get_is_killed_redeem(e: Env) -> bool;
    fn get_is_killed_rebalance(e: Env) -> bool;

    fn get_accumulated_manager_fees(e: Env) -> u128;
    fn get_accumulated_protocol_fees(e: Env) -> u128;
}

// Query Data Structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexInfo {
    pub address: Address,
    pub token_address: Address,
    pub total_shares: u128,
    pub base_nav: u128,
    pub initial_price: u128,
    pub is_public: bool,
    pub manager_fee_fraction: u32,
    pub manager_address: Address,
    pub protocol_fee_recipient: Address,
    pub accumulated_manager_fees: u128,
    pub accumulated_protocol_fees: u128,
    pub last_rebalance_ts: u64,
    pub last_updated_ts: u64,
    pub total_mints: u128,
    pub total_redemptions: u128,
    pub total_fees: u128,
    pub is_killed_mint: bool,
    pub is_killed_redeem: bool,
    pub is_killed_rebalance: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexMetrics {
    pub total_shares: u128,
    pub total_mints: u128,
    pub total_redemptions: u128,
    pub total_fees: u128,
    pub accumulated_manager_fees: u128,
    pub accumulated_protocol_fees: u128,
    pub current_nav: u128,
    pub share_price: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexStatus {
    pub is_killed_mint: bool,
    pub is_killed_redeem: bool,
    pub is_killed_rebalance: bool,
    pub is_public: bool,
    pub can_rebalance: bool,
    pub last_rebalance_ts: u64,
    pub rebalance_threshold: u64,
}

// Rebalancing Data Structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComponentAction {
    Add,
    Remove,
    UpdateWeight,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentUpdate {
    pub token: Address,
    pub new_weight: u128,
    pub action: ComponentAction,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceParams {
    pub component_updates: Vec<ComponentUpdate>,
    pub target_nav: Option<i128>, // Optional NAV target
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentAllocation {
    pub component: Component,
    pub current_balance: u128,
    pub target_balance: u128,
    pub percentage_of_nav: u128, // In basis points
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceStatus {
    pub can_rebalance: bool,
    pub time_until_next_rebalance: u64,
    pub last_rebalance_ts: u64,
    pub rebalance_threshold: u64,
    pub is_public: bool,
    pub authorized_rebalancers: Vec<Address>, // For private indexes
}

// Query Interface
pub trait QueryInterface {
    // Comprehensive index information
    fn get_index_info(e: Env) -> IndexInfo;

    // Component and balance queries
    fn get_all_components(e: Env) -> Map<Address, Component>;
    fn get_component_info(e: Env, token: Address) -> Component;
    fn get_all_component_balances(e: Env) -> Map<Address, u128>;
    fn get_total_index_value(e: Env) -> u128;

    // Financial metrics
    fn get_index_metrics(e: Env) -> IndexMetrics;
    fn get_share_price(e: Env) -> u128;
    fn get_current_nav(e: Env) -> u128;

    // Operational status
    fn get_index_status(e: Env) -> IndexStatus;
    fn can_rebalance(e: Env) -> bool;

    // Rebalancing queries
    fn get_rebalance_status(e: Env) -> RebalanceStatus;
    fn can_address_rebalance(e: Env, caller: Address) -> bool;
    fn get_component_allocation(e: Env) -> Map<Address, ComponentAllocation>;
    fn get_rebalance_authorities(e: Env) -> Vec<Address>;
}

// Privacy Interface - New interface for privacy-aware operations
pub trait PrivacyInterface {
    // Privacy configuration management
    fn set_privacy_config(e: Env, admin: Address, config: IndexPrivacyConfig);
    fn get_privacy_config(e: Env) -> IndexPrivacyConfig;
    
    // Privacy-aware component queries
    fn get_components_for_viewer(e: Env, viewer: Address) -> ComponentView;
    fn get_component_info_for_viewer(e: Env, viewer: Address, token: Address) -> Option<PrivateComponent>;
    
    // Privacy-aware financial queries
    fn get_nav_for_viewer(e: Env, viewer: Address) -> NAVResponse;
    fn get_portfolio_summary_for_viewer(e: Env, viewer: Address) -> PortfolioSummary;
    
    // Component commitment management
    fn commit_component_weights(
        e: Env,
        admin: Address,
        components: Vec<(Address, u128)>,
        salt: u64
    ) -> u32;
    
    fn reveal_component_commitment(
        e: Env,
        admin: Address,
        token: Address,
        weight: u128,
        salt: u64
    ) -> bool;
    
    // Private rebalancing
    fn private_rebalance_commitments(
        e: Env,
        caller: Address,
        new_commitments: Vec<PrivateComponent>,
        commitment_proof: CommitmentProof
    );
    
    // Viewer access management
    fn add_authorized_viewer(e: Env, admin: Address, viewer: Address, access_level: ViewerAccessLevel);
    fn remove_authorized_viewer(e: Env, admin: Address, viewer: Address);
    fn get_viewer_access_level(e: Env, viewer: Address) -> ViewerAccessLevel;
    
    // Privacy utilities
    fn verify_commitment(e: Env, token: Address, weight: u128, salt: u64) -> bool;
    fn encrypt_for_viewer(e: Env, admin: Address, data: Bytes, viewer: Address) -> Bytes;
    fn decrypt_for_viewer(e: Env, viewer: Address, encrypted_data: Bytes) -> Option<Bytes>;
}

// Compliance Interface - For regulatory reporting and auditing
pub trait ComplianceInterface {
    /// Generate compliance report for authorized compliance authorities
    fn generate_compliance_report(e: Env, compliance_authority: Address) -> crate::compliance::ComplianceReport;
    
    /// Get public portfolio summary (no sensitive details)
    fn get_public_portfolio_summary(e: Env) -> crate::compliance::ComplianceReport;
    
    /// Get audit trail for specific time period (compliance authorities only)
    fn get_audit_trail(
        e: Env,
        compliance_authority: Address,
        start_time: u64,
        end_time: u64
    ) -> Vec<crate::compliance::AuditEntry>;
    
    /// Emergency access to decrypt all data (emergency authorities only)
    fn emergency_decrypt_all(
        e: Env,
        emergency_authority: Address
    ) -> Vec<PrivateComponent>;
    
    /// Get risk assessment for compliance purposes
    fn get_risk_assessment(
        e: Env,
        compliance_authority: Address
    ) -> crate::compliance::RiskMetrics;
}

// Zero-Knowledge Proof Interface - For selective revelation and privacy-preserving compliance
pub trait ZKProofInterface {
    /// Generate a range proof for an asset allocation
    /// Allows institutions to prove their allocation is within a range without revealing exact amount
    fn generate_asset_range_proof(
        e: Env,
        asset_owner: Address,
        asset: soroban_sdk::Symbol,
        min_percentage: u128,
        max_percentage: u128,
        salt: u64
    ) -> RangeProof;
    
    /// Verify a range proof for an asset allocation
    fn verify_asset_range_proof(
        e: Env,
        proof: RangeProof
    ) -> bool;
    
    /// Generate a compliance proof showing portfolio meets regulatory requirements
    /// without revealing individual asset weights
    fn generate_compliance_proof(
        e: Env,
        portfolio_owner: Address,
        constraints: Vec<ComplianceConstraint>,
        method: ProofMethod
    ) -> ComplianceProof;
    
    /// Verify a compliance proof
    fn verify_compliance_proof(
        e: Env,
        proof: ComplianceProof
    ) -> bool;
    
    /// Process a selective revelation request
    /// Allows proving allocation ranges for regulatory compliance
    fn process_selective_revelation(
        e: Env,
        request: SelectiveRevelationRequest,
        salt: u64
    ) -> SelectiveRevelationResult;
    
    /// Batch verify multiple ZK proofs for efficiency
    fn batch_verify_proofs(
        e: Env,
        range_proofs: Vec<RangeProof>,
        compliance_proofs: Vec<ComplianceProof>
    ) -> ZKProofStatus;
    
    /// Get ZK proof verification status for a portfolio
    fn get_proof_status(
        e: Env,
        portfolio_owner: Address
    ) -> ZKProofStatus;
    
    /// Selective compliance check - prove specific regulatory constraints
    /// without revealing full portfolio details
    fn prove_regulatory_compliance(
        e: Env,
        institution: Address,
        regulator: Address,
        required_constraints: Vec<ComplianceConstraint>
    ) -> ComplianceProof;
    
    /// Verify that an institution meets specific regulatory requirements
    /// using zero-knowledge proofs
    fn verify_regulatory_compliance(
        e: Env,
        compliance_proof: ComplianceProof,
        required_constraints: Vec<ComplianceConstraint>
    ) -> bool;
}
