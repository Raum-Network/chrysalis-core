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

struct ClaimableBalanceTest<'a> {
    env: Env,
    deposit_address: Address,
    claim_addresses: [Address; 3],
    token: TokenClient<'a>,
    contract: ChrysalisContractClient<'a>,
}

#[test]
fn test_stake_eth() {
    let env = Env::default();
    env.mock_all_auths();
    let user_address = soroban_sdk::Address::generate(&env);
    let (staked_token_address , staked_client) = create_token_contract(&env, &user_address);
    let (steth_address , steth_client) = create_token_contract(&env, &user_address);
    staked_client.mint(&user_address, &1000);
    steth_client.mint(&user_address, &1000);

    // Register the ChrysalisContract
    let contract_id = env.register_contract(None, ChrysalisContract);

    // Create a client to interact with the contract
    let client = ChrysalisContractClient::new(&env, &contract_id);

    // Initialize the contract directly through the client
    client.initialize_contract(&staked_client.address, &staked_client.address);

    // Call stake_eth
    client.stake_eth(&user_address, &500);
    // // Verify staking
    let stake_key = DataKey::Stake(user_address.clone());
    let stake = env.as_contract(&contract_id , || env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap());
    assert_eq!(stake.amount, 500);
}

#[test]
fn test_unstake_eth() {
    let env = Env::default();
    env.mock_all_auths();
    let user_address = soroban_sdk::Address::generate(&env);
    let (staked_token_address , staked_client) = create_token_contract(&env, &user_address);
    let (steth_address , steth_client) = create_token_contract(&env, &user_address);
    staked_client.mint(&user_address, &1000);
    steth_client.mint(&user_address, &1000);
   

    // Register the ChrysalisContract
    let contract_id = env.register_contract(None, ChrysalisContract);

    steth_client.mint(&contract_id, &1000);

    // Create a client to interact with the contract
    let client = ChrysalisContractClient::new(&env, &contract_id);

    // Initialize the contract directly through the client
    client.initialize_contract(&staked_client.address, &staked_client.address);

    // Call stake_eth
    client.stake_eth(&user_address, &500);
    let stake_key = DataKey::Stake(user_address.clone());
    log!(&env, "Staked 500" ,  env.as_contract(&contract_id , || env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap()));
    client.unstake_eth(&user_address, &250);
    log!(&env, "Unstaked 250" , client.get_stake_amount(&user_address));
    
    let stake: Stake = env.as_contract(&contract_id , || env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap());
    log!(&env, "Staked 250" ,  env.as_contract(&contract_id , || env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap()));
    assert_eq!(stake.amount, 250);
}

#[test]
fn test_claim() {
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

    steth_client.mint(&client.address, &1000);
    log!(&env, "Minted 1000" , steth_address.balance(&client.address) , steth_client.address);
    // Initialize the contract directly through the client
    client.initialize_contract(&staked_client.address, &staked_client.address);

    // Simulate minting rewards
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
    let claimed_rewards = client.claim(&user_address);
    log!(&env, "Claimed Rewards: {}", claimed_rewards);

    // Check the final stake to ensure the timestamp was updated
    let stake_key = DataKey::Stake(user_address.clone());
    let updated_stake: Stake = env.as_contract(&contract_id, || {
        env.storage().instance().get::<DataKey, Stake>(&stake_key).unwrap()
    });

    // Validate that the timestamp has been updated
    assert_eq!(updated_stake.timestamp, elapsed_time);

    // Calculate expected rewards manually: reward_rate * amount * duration / 1_000_000
    let reward_rate = 0.05; // 5% per unit of time
    let expected_rewards = (initial_amount as f64 * reward_rate * elapsed_time as f64 / 1_000_000.0) as i128;

    // Log and assert the correct reward amount
    log!(&env, "Claimed Rewards: {}", claimed_rewards , expected_rewards);
    assert_eq!(claimed_rewards, expected_rewards);

    // Check that the stETH tokens were minted correctly
    let final_balance = steth_address.balance(&user_address.clone());
    log!(&env, "Final stETH balance for user: {}", final_balance);

    // Ensure that the user received the expected rewards in stETH
    assert_eq!(final_balance, expected_rewards)
}