#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger};
use soroban_sdk::{symbol_short, token, vec, Address, Env, IntoVal};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &sac.address()),
        token::StellarAssetClient::new(e, &sac.address()),
    )
}

fn create_claimable_balance_contract<'a>(e: &Env) -> ChrysalisContractClient<'a> {
    ChrysalisContractClient::new(e, &e.register_contract(None, ChrysalisContract {}))
}

#[test]
fn test_stake_eth() {
    let env = Env::default();
    env.mock_all_auths();
    let user_address = soroban_sdk::Address::generate(&env);
    let (staked_token_address, staked_client) = create_token_contract(&env, &user_address);
    let (steth_address, steth_client) = create_token_contract(&env, &user_address);

    // Mint ETH to the user
    staked_client.mint(&user_address, &1000);

    // Register the ChrysalisContract
    let contract_id = env.register_contract(None, ChrysalisContract);

    // Mint stETH to the contract
    steth_client.mint(&contract_id, &1000);

    // Create a client to interact with the contract
    let client = ChrysalisContractClient::new(&env, &contract_id);

    // Initialize the contract with staked_token_address and steth_address
    client.initialize_contract(&staked_client.address, &steth_client.address);

    // Call stake_eth
    client.stake_eth(&user_address, &500);

    // Verify staking
    let stake_key = DataKey::Stake(user_address.clone());
    let stake = env.as_contract(&contract_id, || {
        env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap()
    });
    assert_eq!(stake.amount, 500);

    // Check that 500 stETH was transferred to the user
    let steth_balance = steth_address.balance(&user_address);
    assert_eq!(steth_balance, 500);

    // Check that 500 stETH was deducted from the contract
    let contract_steth_balance = steth_address.balance(&contract_id);
    assert_eq!(contract_steth_balance, 500);
}

#[test]
fn test_unstake_eth() {
    let env = Env::default();
    env.mock_all_auths();
    let user_address = soroban_sdk::Address::generate(&env);
    let (staked_token_address, staked_client) = create_token_contract(&env, &user_address);
    let (steth_address, steth_client) = create_token_contract(&env, &user_address);

    // Mint ETH to the user
    staked_client.mint(&user_address, &1000);

    // Register the ChrysalisContract
    let contract_id = env.register_contract(None, ChrysalisContract);

    // Mint stETH to the contract
    steth_client.mint(&contract_id, &1000);

    // Create a client to interact with the contract
    let client = ChrysalisContractClient::new(&env, &contract_id);

    // Initialize the contract
    client.initialize_contract(&staked_client.address, &steth_client.address);

    // Stake 500 ETH
    client.stake_eth(&user_address, &500);

    // Unstake 250 stETH by transferring it back to the contract
    // steth_address.transfer(&user_address, &contract_id, &250);
    client.unstake_eth(&user_address, &250);

    // Verify the new stake amount is 250
    let stake_key = DataKey::Stake(user_address.clone());
    let stake = env.as_contract(&contract_id, || {
        env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap()
    });
    assert_eq!(stake.amount, 250);

    // Verify the user's stETH balance is 250
    let steth_balance = steth_address.balance(&user_address);
    assert_eq!(steth_balance, 250);

    // Verify the user's ETH balance has increased by 250
    let eth_balance = staked_token_address.balance(&user_address);
    assert_eq!(eth_balance, 750);
}

#[test]
fn test_claim() {
    let env = Env::default();
    env.mock_all_auths();
    
    let user_address = soroban_sdk::Address::generate(&env);
    let (staked_token_address, staked_client) = create_token_contract(&env, &user_address);
    let (steth_address, steth_client) = create_token_contract(&env, &user_address);

    // Register the ChrysalisContract
    let contract_id = env.register_contract(None, ChrysalisContract);

    // Mint stETH to the contract to ensure it can distribute rewards
    steth_client.mint(&contract_id, &1000);

    // Create a client to interact with the contract
    let client = ChrysalisContractClient::new(&env, &contract_id);

    // Set the stETH address in the contract's storage
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::StETHAddress, &steth_client.address);
    });

    // Initialize the contract
    client.initialize_contract(&staked_client.address, &steth_client.address);

    // Simulate a stake with a specific timestamp
    let initial_amount: i128 = 1000;
    let initial_timestamp = 0;
    env.as_contract(&contract_id, || {
        env.storage().instance().set(
            &DataKey::Stake(user_address.clone()),
            &Stake {
                amount: initial_amount,
                timestamp: initial_timestamp,
            },
        );
    });

    // Simulate time passage (e.g., 1,000,000 time units)
    let elapsed_time: u64 = 1_000_000;
    env.ledger().set_timestamp(elapsed_time);

    // Call the claim function
    let claimed_rewards = client.claim(&user_address , &5);

    // Calculate expected rewards manually: (amount * reward_rate * duration / 100 / 1_000_000)
    let reward_rate = 5; // 5% per unit of time
    let expected_rewards = (initial_amount * reward_rate * elapsed_time as i128) / 100 / 1_000_000;

    // Log and assert the correct reward amount
    log!(&env, "Claimed Rewards: {}", claimed_rewards);
    assert_eq!(claimed_rewards, expected_rewards);

    // Check that the user received the correct rewards in stETH
    let final_balance = steth_address.balance(&user_address);
    log!(&env, "Final stETH balance for user: {}", final_balance);
    assert_eq!(final_balance, expected_rewards);

    // Ensure that the stake's timestamp was updated correctly
    let stake_key = DataKey::Stake(user_address.clone());
    let updated_stake: Stake = env.as_contract(&contract_id, || {
        env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap()
    });
    // assert_eq!(updated_stake.timestamp, elapsed_time);

    // Ensure that the stake amount is updated after claiming rewards
    assert_eq!(updated_stake.amount, initial_amount + claimed_rewards);
}


#[test]
fn test_vclaim() {
   
    let env = Env::default();
    env.mock_all_auths();
    let user_address = soroban_sdk::Address::generate(&env);
    let (staked_token_address , staked_client) = create_token_contract(&env, &user_address);
    let (steth_address , steth_client) = create_token_contract(&env, &user_address);
    // staked_client.mint(&user_address, &1000);
    

    // Register the ChrysalisContract
    let contract_id = env.register_contract(None, ChrysalisContract);

    // Create a client to interact with the contract
    let client = ChrysalisContractClient::new(&env, &contract_id);

    // Initialize the contract directly through the client
    client.initialize_contract(&staked_client.address, &staked_client.address);

    let initial_amount: i128 = 1000;
    let initial_timestamp = 0;
    env.as_contract(&contract_id, || {
        env.storage().instance().set(
            &DataKey::Stake(user_address.clone()), 
            &Stake { amount: initial_amount, timestamp: initial_timestamp }
        );
    });

    // Simulate some time passage (e.g., 1,000,000 time units)
    let elapsed_time: u64 = 1_000_000; // 1 second in terms of microseconds
    env.ledger().set_timestamp(elapsed_time);

    // Call the claim function and calculate rewards
    let claimed_rewards = client.vclaim(&user_address , &5);
    log!(&env, "Claimed Rewards: {}", claimed_rewards);

    // Check the final stake to ensure the timestamp was updated
    let stake_key = DataKey::Stake(user_address.clone());
    // let updated_stake: Stake = env.as_contract(&contract_id, || {
    //     env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap()
    // });

    // Validate that the timestamp has been updated
    // assert_eq!(updated_stake.timestamp, elapsed_time);

    // Calculate expected rewards manually: reward_rate * amount * duration / 1_000_000
    let reward_rate: i128 = 5; // 5% per unit of time
    let expected_rewards: i128 = (initial_amount * reward_rate * elapsed_time as i128) / 100 / 1_000_000;

    // Log and assert the correct reward amount
    log!(&env, "Claimed Rewards: {}", claimed_rewards , expected_rewards);
    assert_eq!(claimed_rewards, expected_rewards);
}