pub fn calculate_fee_split(
    amount: u128,
    protocol_fee_bps: u32,
    manager_fee_bps: u32,
) -> (u128, u128, u128) {
    if amount == 0 || (protocol_fee_bps == 0 && manager_fee_bps == 0) {
        return (0, 0, 0);
    }

    let protocol_fee = amount.saturating_mul(protocol_fee_bps as u128) / 10_000;
    let manager_fee = amount.saturating_mul(manager_fee_bps as u128) / 10_000;

    let total_fee = protocol_fee.saturating_add(manager_fee);

    (total_fee, protocol_fee, manager_fee)
}
