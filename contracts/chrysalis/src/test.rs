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
