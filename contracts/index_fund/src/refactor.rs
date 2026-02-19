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

    // Create component
    let component = Component {
        asset: Symbol::new(e, "TOKEN"),
        weight: update.new_weight,
        oracle: update.oracle,
        adapter_type: update.adapter_type.clone(),
        adapter: update.adapter.clone(),
    };
    crate::storage::set_component(e, update.token.clone(), component);
    crate::storage::add_component_to_registry(e, update.token.clone());

    // Get component balance for NAV impact calculation
    let initial_balance =
        crate::storage::get_component_balance_safe(e, update.token.clone()).unwrap_or(0);

    Events::new(e).component_added(
        current_time,
        admin.clone(),
        update.token.clone(),
        update.new_weight,
        initial_balance,
        0, // TODO: Calculate actual NAV impact
    );
}

pub fn remove_component(e: &Env, update: ComponentUpdate, current_time: u64) {
    validate_component(e, update.token.clone());

    // Get component info before removing
    let component = crate::storage::get_component(e, update.token.clone()); // This will panic if not found
    let old_weight = component.weight;
    let final_balance =
        crate::storage::get_component_balance_safe(e, update.token.clone()).unwrap_or(0);

    crate::storage::remove_component(e, update.token.clone());

    Events::new(e).component_removed(
        current_time,
        admin.clone(),
        update.token.clone(),
        final_balance,
        final_balance, // proceeds_distributed (approximation)
        0,             // TODO: Calculate actual NAV impact
    );
}

pub fn update_component(e: &Env, update: ComponentUpdate, current_time: u64) {
    validate_component(e, update.token.clone());

    // Get component
    let mut component = crate::storage::get_component(e, update.token.clone());

    // Update
    if update.action == ComponentAction::UpdateWeight {
        let old_weight = component.weight;
        component.weight = update.new_weight;

        Events::new(e).component_weight_updated(
            update.token.clone(),
            old_weight,
            update.new_weight,
        );
    } else if update.action == ComponentAction::UpdateAdapter {
        let old_adapter = component.adapter;
        component.adapter = update.adapter;

        Events::new(e).component_adapter_updated(
            update.token.clone(),
            old_adapter,
            update.new_adapter,
        );
    } else if update.action == ComponentAction::UpdateOracle {
        let old_oracle = component.oracle;
        component.oracle = update.oracle;

        Events::new(e).component_oracle_updated(
            update.token.clone(),
            old_oracle,
            update.new_oracle,
        );
    }

    // Save
    crate::storage::set_component(e, update.token.clone(), component);
}

pub fn handle_update(e: &Env, update: ComponentUpdate, current_time: u64) {
    match update.action {
        ComponentAction::Add => add_component(e, update, current_time),
        ComponentAction::Remove => remove_component(e, update, current_time),
        ComponentAction::UpdateWeight
        | ComponentAction::UpdateAdapter
        | ComponentAction::UpdateOracle => update_component(e, update, current_time),
    }
}

pub fn refactor(e: &Env, authority: Address, params: RefactorParams, current_time: u64) {
    // Validate and execute component updates (without swaps)
    let len = params.component_updates.len();
    for i in 0..len {
        handle_update(e, params.component_updates.get_unchecked(i), current_time);
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
