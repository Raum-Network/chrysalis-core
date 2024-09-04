#![no_std]

mod contract_interface;
mod testold;
mod test;

use core::fmt::Display;

use soroban_sdk::{contract, contractimpl, contracttype, events, log, Address, Env, IntoVal, Symbol, Vec};
use contract_interface::ChrysalisContractTrait;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Stake {
    amount: i128,
    timestamp: u64,
}

impl Default for Stake {
    fn default() -> Self {
        Stake {
            amount: 0,
            timestamp: 0,
        }
    }
}


#[contract]
struct ChrysalisContract;

#[derive(Clone )]
#[contracttype]
enum DataKey {
    Stake(Address),
    StakedTokenAddress,
    StETHAddress,
}

#[contractimpl]
impl ChrysalisContractTrait for ChrysalisContract {
    
    fn initialize_contract(env: Env, staked_token_address: Address, steth_address: Address) {
        env.storage().instance().set(&DataKey::StakedTokenAddress, &staked_token_address);
        env.storage().instance().set(&DataKey::StETHAddress, &steth_address);
    }

    fn stake_eth(env: Env, user: Address, amount: i128) {
        user.require_auth();
        let staked_token_address: Address = env.storage().instance().get(&DataKey::StakedTokenAddress).unwrap();
        let steth_address: Address = env.storage().instance().get(&DataKey::StETHAddress).unwrap();

        let balance: i128 = env.invoke_contract(
            &staked_token_address,
            &Symbol::new(&env.clone(), "balance"),
            (&user,).into_val(&env),
        );

        log!(&env, "Balance: {}", balance);

        assert!(balance >= amount, "Insufficient balance");

        env.invoke_contract::<()>(
            &staked_token_address,
            &Symbol::new(&env.clone(), "transfer"),
            Vec::from_array(
                &env,
                [
                    user.into_val(&env.clone()),
                    env.current_contract_address().into_val(&env.clone()),
                    amount.into_val(&env.clone()),
                ]
            ),
        );


        env.invoke_contract::<()>(
            &steth_address,
            &Symbol::new(&env.clone(), "mint"),
            Vec::from_array(
                &env,
                [
                    // env.current_contract_address().into_val(&env.clone()),
                    user.into_val(&env.clone()),
                    amount.into_val(&env.clone()),
                ]
            ),
        );

        let mut stakes: Stake = env.storage().instance().get(&user.clone()).unwrap_or_default();

        stakes.amount += amount;
        stakes.timestamp = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Stake(user), &stakes);
    }

    
    fn unstake_eth(env: Env, user: Address, amount: i128) -> () {
        let staked_token_address: Address = env.storage().instance().get(&DataKey::StakedTokenAddress).unwrap();
        let steth_address: Address = env.storage().instance().get(&DataKey::StETHAddress).unwrap();

        env.invoke_contract::<()>(
            &steth_address,
            &Symbol::new(&env.clone(), "burn"),
            Vec::from_array(
                &env,
                [
                    user.into_val(&env.clone()),
                    amount.into_val(&env.clone()),
                ]
            ),
        );

        env.invoke_contract::<()>(
            &staked_token_address,
            &Symbol::new(&env.clone(), "transfer"),
            Vec::from_array(
                &env,
                [
                    env.current_contract_address().into_val(&env.clone()),
                    user.into_val(&env.clone()),
                    amount.into_val(&env.clone()),
                ]
            ),
        );

        let mut stakes: Stake = env.storage().instance().get(&user.clone()).unwrap_or_default();
        stakes.amount -= amount;
        env.storage().instance().set(&DataKey::Stake(user), &stakes);
    }

    fn get_stake_amount(env: Env, user: Address) -> i128 {
        let stakes: Stake = env.storage().instance().get(&user.clone()).unwrap_or_default();
        stakes.amount
    }

    fn claim(env: Env, user: Address) -> i128 {
        let mut stakes: Stake = env.storage().instance().get(&user.clone()).unwrap_or_default();
        
        // Assuming simple interest for rewards calculation:
        let reward_rate = 0.05; // 5% reward rate per unit of time
        let current_timestamp = env.ledger().timestamp();
        let staking_duration = current_timestamp - stakes.timestamp;

        let rewards = (stakes.amount as f64 * reward_rate * staking_duration as f64 / 1_000_000.0) as i64;
        stakes.timestamp = current_timestamp;

        // Transfer rewards to the user
        let steth_address: Address = env.storage().instance().get(&DataKey::StETHAddress).unwrap();
        env.invoke_contract::<()>(
            &steth_address,
            &Symbol::new(&env.clone(), "mint"),
            Vec::from_array(
                &env,
                [
                    env.current_contract_address().into_val(&env.clone()),
                    user.into_val(&env.clone()),
                    rewards.into_val(&env.clone()),
                ]
            ),
        );

        // Update the stakes
        env.storage().instance().set(&DataKey::Stake(user), &stakes);

        rewards.into()
    }

    // New Verify Claim Function
    fn vclaim(env: Env, user: Address) -> i128 {
        let stakes: Stake = env.storage().instance().get(&user.clone()).unwrap_or_default();

        // Assuming simple interest for rewards calculation:
        let reward_rate = 0.05; // 5% reward rate per unit of time
        let current_timestamp = env.ledger().timestamp();
        let staking_duration = current_timestamp - stakes.timestamp;

        let rewards = (stakes.amount as f64 * reward_rate * staking_duration as f64 / 1_000_000.0) as i64;
        rewards.into()
    }
}
