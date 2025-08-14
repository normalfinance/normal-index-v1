use soroban_sdk::{contracttype, Address, Bytes, Symbol, Vec};

/// Privacy mode for individual components
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComponentPrivacyMode {
    /// Public mode - weight directly visible (current behavior)
    Public(u128),
    /// Private mode - only hash commitment stored
    Private,
    /// Authorized mode - encrypted for specific viewers
    Authorized(Vec<Address>),
}

/// Privacy mode for entire index
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexPrivacyMode {
    /// All components public
    Public,
    /// All components private with hash commitments
    Private,
    /// Mixed mode - some components public, some private
    Mixed,
    /// Authorized access - encrypted for specific viewers
    Authorized,
}

/// Access levels for viewers
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewerAccessLevel {
    /// Public access - only aggregate metrics
    Public,
    /// Investor access - performance metrics, no strategy details
    Investor,
    /// Authorized access - can decrypt component weights
    Authorized,
    /// Compliance access - full regulatory view
    Compliance,
    /// Emergency access - recovery and emergency operations
    Emergency,
}

/// Enhanced component structure with privacy support
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateComponent {
    /// Asset symbol - always visible for compliance
    pub asset: Symbol,
    /// Hash commitment of weight (for private mode)
    pub weight_commitment: Option<Bytes>,
    /// Encrypted weight for authorized viewers
    pub encrypted_weight: Option<Bytes>,
    /// Privacy mode for this component
    pub privacy_mode: ComponentPrivacyMode,
}

/// Privacy configuration for an index
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexPrivacyConfig {
    /// Overall privacy mode for the index
    pub privacy_mode: IndexPrivacyMode,
    /// Addresses authorized to view private data
    pub authorized_viewers: Vec<Address>,
    /// Compliance authorities with full access
    pub compliance_authorities: Vec<Address>,
    /// Emergency access addresses
    pub emergency_access: Vec<Address>,
    /// Master encryption key hash for verification
    pub encryption_key_hash: Option<Bytes>,
}

/// Response types for component queries based on access level
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComponentView {
    /// Full component details (for authorized viewers)
    Full(Vec<PrivateComponent>),
    /// Only asset symbols, no weights
    AssetsOnly(Vec<Symbol>),
    /// Aggregate metrics only
    Aggregate(AggregateMetrics),
    /// No access
    Unauthorized,
}

/// Aggregate portfolio metrics for public view
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateMetrics {
    /// Total number of assets
    pub asset_count: u32,
    /// Total portfolio value
    pub total_value: u128,
    /// Diversification score (without revealing specific allocations)
    pub diversification_score: u32,
}

/// NAV response based on viewer access level
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NAVResponse {
    /// Public NAV only
    Public(u128),
    /// Detailed NAV with performance breakdown
    Detailed(DetailedNAV),
    /// Aggregate NAV with limited details
    Aggregate(u128),
}

/// Detailed NAV information for authorized viewers
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetailedNAV {
    /// Current NAV
    pub current_nav: u128,
    /// NAV breakdown by component (for authorized viewers)
    pub component_values: Vec<(Symbol, u128)>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceMetrics {
    /// Total return percentage (basis points)
    pub total_return_bps: i128,
    /// Annualized return percentage (basis points)
    pub annualized_return_bps: i128,
    /// Sharpe ratio (scaled by 1000)
    pub sharpe_ratio: i128,
}


/// Portfolio summary for different access levels
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioSummary {
    /// Total portfolio value
    pub total_value: u128,
    /// Number of assets (always visible)
    pub asset_count: u32,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Risk metrics (aggregated)
    pub risk_score: u32,
    /// Component details (if authorized)
    pub components: Option<Vec<PrivateComponent>>,
}








