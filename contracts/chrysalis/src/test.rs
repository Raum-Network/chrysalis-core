#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Env as _};
use soroban_sdk::{Address, Env, IntoVal, Symbol};

// Mock token contract to simulate the custom token
mod token {
    soroban_sdk::contractimport!(
        file = "../path_to_your_mock_token_contract.wasm"
    );
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract = StakingContract {};
    
    let staked_token_address = Address::random(&env);
    let steth_address = Address::random(&env);
    
    contract.initialize(env.clone(), staked_token_address.clone(), steth_address.clone());
    
    let stored_staked_token_address: Address = env.storage().get_unchecked(DataKey::StakedTokenAddress).unwrap();
    let stored_steth_address: Address = env.storage().get_unchecked(DataKey::StETHAddress).unwrap();
    
    assert_eq!(stored_staked_token_address, staked_token_address);
    assert_eq!(stored_steth_address, steth_address);
}

#[test]
fn test_stake() {
    let env = Env::default();
    let contract = StakingContract {};

    let user = Address::random(&env);
    let staked_token_address = Address::random(&env);
    let steth_address = Address::random(&env);
    
    // Initialize the contract
    contract.initialize(env.clone(), staked_token_address.clone(), steth_address.clone());

    // Mock the custom token balance and transfer
    let mock_token_client = token::Client::new(&env, &staked_token_address);

    mock_token_client.mock_balance(&user, 1000);  // Mock user having 1000 tokens
    mock_token_client.mock_transfer(&user, &env.current_contract_address(), 500);  // Mock transfer of 500 tokens
    
    // Perform staking
    contract.stake(env.clone(), user.clone(), 500);
    
    // Check if the staked amount is recorded correctly
    let stake_amount = contract.get_stake(env.clone(), user.clone());
    assert_eq!(stake_amount, 500);
    
    // Check if stETH tokens were minted
    let steth_balance: i64 = env.invoke_contract(
        &steth_address,
        &symbol!("balance_of"),
        (&user,).into_val(&env),
    );
    assert_eq!(steth_balance, 500);
}

#[test]
fn test_unstake() {
    let env = Env::default();
    let contract = StakingContract {};

    let user = Address::random(&env);
    let staked_token_address = Address::random(&env);
    let steth_address = Address::random(&env);
    
    // Initialize the contract
    contract.initialize(env.clone(), staked_token_address.clone(), steth_address.clone());

    // Mock the custom token balance, transfer, and minting of stETH
    let mock_token_client = token::Client::new(&env, &staked_token_address);
    mock_token_client.mock_balance(&user, 1000);
    mock_token_client.mock_transfer(&user, &env.current_contract_address(), 500);

    let mock_steth_client = token::Client::new(&env, &steth_address);
    mock_steth_client.mock_mint(&env.current_contract_address(), &user, 500);
    
    // Stake tokens
    contract.stake(env.clone(), user.clone(), 500);

    // Unstake tokens
    contract.unstake(env.clone(), user.clone(), 500);
    
    // Check if staked amount is reduced
    let stake_amount = contract.get_stake(env.clone(), user.clone());
    assert_eq!(stake_amount, 0);
    
    // Check if stETH tokens were burned
    let steth_balance: i64 = env.invoke_contract(
        &steth_address,
        &symbol!("balance_of"),
        (&user,).into_val(&env),
    );
    assert_eq!(steth_balance, 0);

    // Check if original tokens were transferred back to the user
    let user_balance: i64 = env.invoke_contract(
        &staked_token_address,
        &symbol!("balance_of"),
        (&user,).into_val(&env),
    );
    assert_eq!(user_balance, 1000);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_stake_insufficient_balance() {
    let env = Env::default();
    let contract = StakingContract {};

    let user = Address::random(&env);
    let staked_token_address = Address::random(&env);
    let steth_address = Address::random(&env);
    
    // Initialize the contract
    contract.initialize(env.clone(), staked_token_address.clone(), steth_address.clone());

    // Mock the custom token balance
    let mock_token_client = token::Client::new(&env, &staked_token_address);
    mock_token_client.mock_balance(&user, 100);  // User only has 100 tokens
    
    // Attempt to stake more tokens than available
    contract.stake(env.clone(), user.clone(), 500);
}

#[test]
fn test_unstake_partial_amount() {
    let env = Env::default();
    let contract = StakingContract {};

    let user = Address::random(&env);
    let staked_token_address = Address::random(&env);
    let steth_address = Address::random(&env);
    
    // Initialize the contract
    contract.initialize(env.clone(), staked_token_address.clone(), steth_address.clone());

    // Mock the custom token balance and transfer
    let mock_token_client = token::Client::new(&env, &staked_token_address);
    mock_token_client.mock_balance(&user, 1000);
    mock_token_client.mock_transfer(&user, &env.current_contract_address(), 500);
    
    // Stake tokens
    contract.stake(env.clone(), user.clone(), 500);

    // Unstake partial amount
    contract.unstake(env.clone(), user.clone(), 200);
    
    // Check if the staked amount is reduced accordingly
    let stake_amount = contract.get_stake(env.clone(), user.clone());
    assert_eq!(stake_amount, 300);

    // Check if the corresponding stETH tokens were burned
    let steth_balance: i64 = env.invoke_contract(
        &steth_address,
        &symbol!("balance_of"),
        (&user,).into_val(&env),
    );
    assert_eq!(steth_balance, 300);

    // Check if the original tokens were transferred back to the user
    let user_balance: i64 = env.invoke_contract(
        &staked_token_address,
        &symbol!("balance_of"),
        (&user,).into_val(&env),
    );
    assert_eq!(user_balance, 700);  // 1000 - 500 + 200 = 700
}
