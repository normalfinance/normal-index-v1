use soroban_sdk::{panic_with_error, Address, Env, Symbol};
use types::component::{Component, ComponentAction, ComponentUpdate, RefactorParams};

use crate::events::Events;
use crate::events::IndexEvents;

use crate::errors::IndexFundError;

fn validate_component(e: &Env, token: Address) {
    // Check if component exists first
    let component_exists = crate::storage::get_component_safe(e, token).is_some();

    if !component_exists {
        panic_with_error!(e, IndexFundError::ComponentNotFound);
    }
}

pub fn add_component(e: &Env, update: ComponentUpdate, current_time: u64) {
    // Check if component already exists
    if crate::storage::get_component_safe(e, update.token.clone()).is_some() {
        panic_with_error!(e, IndexFundError::InvalidComponentAction);
    }

    let (new_weight, new_oracle, new_adapter) =
        match (update.new_weight, update.new_oracle, update.new_adapter) {
            (Some(weight), Some(oracle), Some(adapter)) => (weight, oracle, adapter),
            _ => panic_with_error!(e, IndexFundError::InvalidComponentAction),
        };

    // Create component
    let component = Component {
        asset: Symbol::new(e, "TOKEN"),
        weight: new_weight,
        oracle: new_oracle,
        adapter: new_adapter,
    };
    crate::storage::set_component(e, update.token.clone(), component);
    crate::storage::add_component_to_registry(e, update.token.clone());

    // Get component balance for NAV impact calculation
    let initial_balance =
        crate::storage::get_component_balance_safe(e, update.token.clone()).unwrap_or(0);

    Events::new(e).component_added(
        current_time,
        e.current_contract_address(),
        update.token.clone(),
        new_weight,
        initial_balance,
        0, // TODO: Calculate actual NAV impact
    );
}

pub fn remove_component(e: &Env, authority: Address, update: ComponentUpdate, current_time: u64) {
    validate_component(e, update.token.clone());

    // Get component info before removing
    let component = crate::storage::get_component(e, update.token.clone()); // This will panic if not found
    let old_weight = component.weight;
    let final_balance =
        crate::storage::get_component_balance_safe(e, update.token.clone()).unwrap_or(0);

    crate::storage::remove_component(e, update.token.clone());

    Events::new(e).component_removed(
        current_time,
        authority.clone(),
        update.token.clone(),
        final_balance,
        final_balance, // proceeds_distributed (approximation)
        0,             // TODO: Calculate actual NAV impact
    );
}

pub fn update_component(e: &Env, authority: Address, update: ComponentUpdate, current_time: u64) {
    validate_component(e, update.token.clone());

    // Get component
    let mut component = crate::storage::get_component(e, update.token.clone());

    if let Some(new_weight) = update.new_weight {
        let old_weight = component.weight;
        let balance =
            crate::storage::get_component_balance_safe(e, update.token.clone()).unwrap_or(0);
        component.weight = new_weight;
        Events::new(e).component_weight_updated(
            current_time,
            authority.clone(),
            update.token.clone(),
            old_weight,
            new_weight,
            balance,
            balance,
            0,
        );
    }

    if let Some(new_adapter) = update.new_adapter {
        let old_adapter = component.adapter.clone();
        component.adapter = new_adapter.clone();
        Events::new(e).component_adapter_updated(
            current_time,
            authority.clone(),
            update.token.clone(),
            old_adapter,
            new_adapter,
        );
    }

    if let Some(new_oracle) = update.new_oracle {
        let old_oracle = component.oracle.clone();
        component.oracle = new_oracle.clone();
        Events::new(e).component_oracle_updated(
            current_time,
            authority.clone(),
            update.token.clone(),
            old_oracle,
            new_oracle,
        );
    }

    // Save
    crate::storage::set_component(e, update.token.clone(), component);
}

pub fn handle_update(e: &Env, authority: Address, update: ComponentUpdate, current_time: u64) {
    match update.action {
        ComponentAction::Add => add_component(e, update, current_time),
        ComponentAction::Remove => remove_component(e, authority, update, current_time),
        ComponentAction::UpdateWeight
        | ComponentAction::UpdateAdapter
        | ComponentAction::UpdateOracle => update_component(e, authority, update, current_time),
    }
}

pub fn refactor(e: &Env, authority: Address, params: RefactorParams, current_time: u64) {
    // Validate and execute component updates (without swaps)
    let len = params.component_updates.len();
    for i in 0..len {
        handle_update(
            e,
            authority.clone(),
            params.component_updates.get_unchecked(i),
            current_time,
        );
    }

    // Validate that final weights sum to 10000
    // Calculate by iterating registry and getting components directly (avoiding get_all_components Map issues)
    let component_registry = crate::storage::get_component_registry(e);
    let registry_len = component_registry.len();

    // If no components, weights should sum to 0 (valid empty state)
    if registry_len == 0 {
        return;
    }

    let mut total_weight = 0u128;
    for i in 0..registry_len {
        let token_address = component_registry.get_unchecked(i);

        // Get component directly from storage instead of using Map
        if let Some(component) = crate::storage::get_component_safe(e, token_address.clone()) {
            total_weight += component.weight;
        }
    }

    // Validate that final weights sum to 10000
    if total_weight != 10000 {
        panic_with_error!(e, IndexFundError::InvalidWeightSum);
    }
}
