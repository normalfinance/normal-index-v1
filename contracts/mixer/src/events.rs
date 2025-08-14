use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};
use crate::types::{PortfolioTier, WithdrawalStatus};

pub trait MixerEvents {
    /// Emitted when an index deposits to the mixer
    fn deposit_to_mixer(
        &self,
        index_address: Address,
        total_usd_value: u128,
        new_tier: Option<PortfolioTier>,
        deposit_id: u32,
    );

    /// Emitted when a withdrawal is requested
    fn withdrawal_requested(
        &self,
        index_address: Address,
        usd_amount: u128,
        preferred_assets: Vec<Symbol>,
        request_id: u32,
    );

    /// Emitted when a withdrawal is executed
    fn withdrawal_executed(
        &self,
        index_address: Address,
        usd_amount: u128,
        assets_transferred: Vec<(Symbol, u128)>,
        request_id: u32,
    );

    /// Emitted when an index's tier is upgraded
    fn tier_upgraded(
        &self,
        index_address: Address,
        old_tier: PortfolioTier,
        new_tier: PortfolioTier,
        new_allowance: u128,
    );

    /// Emitted when withdrawal status changes
    fn withdrawal_status_changed(
        &self,
        request_id: u32,
        index_address: Address,
        old_status: WithdrawalStatus,
        new_status: WithdrawalStatus,
    );

    /// Emitted when mixer configuration is updated
    fn mixer_config_updated(&self, admin: Address);

    /// Emitted when an index is authorized for mixer
    fn index_authorized(&self, index_address: Address, admin: Address);

    /// Emitted when an index is deauthorized from mixer
    fn index_deauthorized(&self, index_address: Address, admin: Address);
}

pub struct MixerContract;

impl MixerEvents for MixerContract {
    fn deposit_to_mixer(
        &self,
        index_address: Address,
        total_usd_value: u128,
        new_tier: Option<PortfolioTier>,
        deposit_id: u32,
    ) {
        let env = self.env();
        env.events().publish(
            (
                Symbol::new(&env, "deposit_to_mixer"),
                index_address,
                deposit_id,
            ),
            (total_usd_value, new_tier),
        );
    }

    fn withdrawal_requested(
        &self,
        index_address: Address,
        usd_amount: u128,
        preferred_assets: Vec<Symbol>,
        request_id: u32,
    ) {
        let env = self.env();
        env.events().publish(
            (
                Symbol::new(&env, "withdrawal_requested"),
                index_address,
                request_id,
            ),
            (usd_amount, preferred_assets),
        );
    }

    fn withdrawal_executed(
        &self,
        index_address: Address,
        usd_amount: u128,
        assets_transferred: Vec<(Symbol, u128)>,
        request_id: u32,
    ) {
        let env = self.env();
        env.events().publish(
            (
                Symbol::new(&env, "withdrawal_executed"),
                index_address,
                request_id,
            ),
            (usd_amount, assets_transferred),
        );
    }

    fn tier_upgraded(
        &self,
        index_address: Address,
        old_tier: PortfolioTier,
        new_tier: PortfolioTier,
        new_allowance: u128,
    ) {
        let env = self.env();
        env.events().publish(
            (
                Symbol::new(&env, "tier_upgraded"),
                index_address,
            ),
            (old_tier, new_tier, new_allowance),
        );
    }

    fn withdrawal_status_changed(
        &self,
        request_id: u32,
        index_address: Address,
        old_status: WithdrawalStatus,
        new_status: WithdrawalStatus,
    ) {
        let env = self.env();
        env.events().publish(
            (
                Symbol::new(&env, "withdrawal_status_changed"),
                request_id,
                index_address,
            ),
            (old_status, new_status),
        );
    }

    fn mixer_config_updated(&self, admin: Address) {
        let env = self.env();
        env.events().publish(
            (Symbol::new(&env, "mixer_config_updated"), admin),
            (),
        );
    }

    fn index_authorized(&self, index_address: Address, admin: Address) {
        let env = self.env();
        env.events().publish(
            (
                Symbol::new(&env, "index_authorized"),
                index_address,
                admin,
            ),
            (),
        );
    }

    fn index_deauthorized(&self, index_address: Address, admin: Address) {
        let env = self.env();
        env.events().publish(
            (
                Symbol::new(&env, "index_deauthorized"),
                index_address,
                admin,
            ),
            (),
        );
    }
}

impl MixerContract {
    pub fn env(&self) -> Env {
        soroban_sdk::Env::default()
    }
}