use soroban_sdk::{contract::Contract, contract::TestContract, testutils::{Address as TestAddress, Env}};
use soroban_sdk::{symbol, Address, Vec, Env, IntoVal};

use super::ChrysalisContract;
use super::Stake;

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let contract = ChrysalisContract::deploy(&env);

    let staked_token_address = TestAddress::new();
    let steth_address = TestAddress::new();
    contract.initialize_contract(&env, staked_token_address.clone(), steth_address.clone());

    assert_eq!(contract.storage().get(&DataKey::StakedTokenAddress).unwrap(), staked_token_address);
    assert_eq!(contract.storage().get(&DataKey::StETHAddress).unwrap(), steth_address);
}

#[test]
fn test_stake_eth() {
    let env = Env::default();
    let contract = ChrysalisContract::deploy(&env);

    let staked_token_address = TestAddress::new();
    let steth_address = TestAddress::new();
    contract.initialize_contract(&env, staked_token_address.clone(), steth_address.clone());

    let user = TestAddress::new();
    let amount = 1000;

    // Simulate token transfer to the contract
    env.invoke_contract::<()>(
        &staked_token_address,
        &symbol!("transfer"),
        (user.clone(), contract.address(), amount).into_val(&env),
    );

    contract.stake_eth(&env, user.clone(), amount);

    let stake = contract.get_stake_amount(&env, user.clone());
    assert_eq!(stake, amount);
}

#[test]
fn test_unstake_eth() {
    let env = Env::default();
    let contract = ChrysalisContract::deploy(&env);

    let staked_token_address = TestAddress::new();
    let steth_address = TestAddress::new();
    contract.initialize_contract(&env, staked_token_address.clone(), steth_address.clone());

    let user = TestAddress::new();
    let amount = 1000;

    // Simulate staking
    env.invoke_contract::<()>(
        &staked_token_address,
        &symbol!("transfer"),
        (user.clone(), contract.address(), amount).into_val(&env),
    );

    contract.stake_eth(&env, user.clone(), amount);

    // Simulate passage of time
    env.ledger().advance_time(3600); // 1 hour later

    // Unstake tokens
    contract.unstake_eth(&env, user.clone(), amount);

    let stake = contract.get_stake_amount(&env, user.clone());
    assert_eq!(stake, 0);
}

#[test]
fn test_claim() {
    let env = Env::default();
    let contract = ChrysalisContract::deploy(&env);

    let staked_token_address = TestAddress::new();
    let steth_address = TestAddress::new();
    contract.initialize_contract(&env, staked_token_address.clone(), steth_address.clone());

    let user = TestAddress::new();
    let amount = 1000;

    // Simulate staking
    env.invoke_contract::<()>(
        &staked_token_address,
        &symbol!("transfer"),
        (user.clone(), contract.address(), amount).into_val(&env),
    );

    contract.stake_eth(&env, user.clone(), amount);

    // Simulate passage of time
    let initial_timestamp = env.ledger().timestamp();
    env.ledger().advance_time(3600); // 1 hour later
    let current_timestamp = env.ledger().timestamp();

    // Call claim function
    let claimed_amount = contract.claim(&env, user.clone());

    // Calculate expected rewards (simple interest example, 5% per hour)
    let reward_rate = 0.05; // 5% per hour
    let staking_duration = current_timestamp - initial_timestamp;
    let expected_rewards = (amount as f64 * reward_rate * staking_duration as f64 / 3600.0) as i64;

    assert_eq!(claimed_amount, expected_rewards, "Claimed amount does not match expected rewards");

    // Verify new stake amount
    let stake_amount = contract.get_stake_amount(&env, user.clone());
    assert_eq!(stake_amount, amount, "Stake amount should remain unchanged after claiming");
}

#[test]
fn test_vclaim() {
    let env = Env::default();
    let contract = ChrysalisContract::deploy(&env);

    let staked_token_address = TestAddress::new();
    let steth_address = TestAddress::new();
    contract.initialize_contract(&env, staked_token_address.clone(), steth_address.clone());

    let user = TestAddress::new();
    let amount = 1000;

    // Simulate staking
    env.invoke_contract::<()>(
        &staked_token_address,
        &symbol!("transfer"),
        (user.clone(), contract.address(), amount).into_val(&env),
    );

    contract.stake_eth(&env, user.clone(), amount);

    // Simulate passage of time
    let initial_timestamp = env.ledger().timestamp();
    env.ledger().advance_time(3600); // 1 hour later
    let current_timestamp = env.ledger().timestamp();

    // Call vclaim function
    let claimable_amount = contract.vclaim(&env, user.clone());

    // Calculate expected rewards (simple interest example, 5% per hour)
    let reward_rate = 0.05; // 5% per hour
    let staking_duration = current_timestamp - initial_timestamp;
    let expected_rewards = (amount as f64 * reward_rate * staking_duration as f64 / 3600.0) as i64;

    assert_eq!(claimable_amount, expected_rewards, "Claimable amount does not match expected rewards");
}
