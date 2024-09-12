#![no_std]

mod contract_interface;
mod test;

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

        // Transfer staked ETH from user to contract
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

        // Transfer stETH from the contract's address to the user (instead of minting)
        env.invoke_contract::<()>(
            &steth_address,
            &Symbol::new(&env.clone(), "transfer"),
            Vec::from_array(
                &env,
                [
                    env.current_contract_address().into_val(&env.clone()),  // From the contract's balance
                    user.into_val(&env.clone()),                             // To the user
                    amount.into_val(&env.clone()),
                ]
            ),
        );

        // Update user's stake information
        let mut stakes: Stake = env.storage().instance().get(&user.clone()).unwrap_or_default();
        
        stakes.amount += amount;
        stakes.timestamp = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Stake(user), &stakes);
    }

    
    fn unstake_eth(env: Env, user: Address, amount: i128) -> () {
        user.require_auth();
    
        // Retrieve staked token addresses
        let staked_token_address: Address = env.storage().instance().get(&DataKey::StakedTokenAddress).unwrap();
        let steth_address: Address = env.storage().instance().get(&DataKey::StETHAddress).unwrap();
    
        // Get the user's current stake
        let mut stakes: Stake = env.storage().instance().get(&DataKey::Stake(user.clone())).unwrap_or_default();
    
        // Ensure sufficient stake balance to unstake
        if stakes.amount < amount {
            panic!("Insufficient balance to unstake!");
        }
    
        // Transfer stETH back from user to contract (instead of burning)
        env.invoke_contract::<()>(
            &steth_address,
            &Symbol::new(&env.clone(), "transfer"),
            Vec::from_array(
                &env,
                [
                    user.into_val(&env.clone()),                             // From the user
                    env.current_contract_address().into_val(&env.clone()),  // Back to the contract
                    amount.into_val(&env.clone()),
                ]
            ),
        );
    
        // Transfer staked ETH to user
        env.invoke_contract::<()>(
            &staked_token_address,
            &Symbol::new(&env.clone(), "transfer"),
            Vec::from_array(
                &env,
                [
                    env.current_contract_address().into_val(&env.clone()),  // From the contract
                    user.into_val(&env.clone()),                             // To the user
                    amount.into_val(&env.clone()),
                ]
            ),
        );
    
        // Subtract the unstaked amount from the user's stake
        stakes.amount -= amount;
        log!(&env, "Updated Balance: {}", stakes.amount);
    
        // Update storage with the new stake amount
        env.storage().instance().set(&DataKey::Stake(user.clone()), &stakes);
    }
    

    fn get_stake_amount(env: Env, user: Address) -> i128 {
        let stakes: Stake = env.storage().instance().get(&user.clone()).unwrap_or_default();
        stakes.amount
    }

    fn claim(env: Env, user: Address) -> i128 {
        // Ensure the user is authenticated
        user.require_auth();
    
        // Fetch the user's current stake, or initialize to default if it doesn't exist
        let mut stakes: Stake = env.storage().instance().get(&DataKey::Stake(user.clone())).unwrap_or_default();
        
        // Simple interest for rewards calculation: 5% reward rate
        let reward_rate: f64 = 0.05;
        let current_timestamp: u64 = env.ledger().timestamp();
        let staking_duration: u64 = current_timestamp - stakes.timestamp;
    
        // Calculate rewards
        let rewards: i128 = ((stakes.amount as f64 * reward_rate * staking_duration as f64 / 1_000_000.0) as i64).into();
        
        // Update the stake's timestamp to the current ledger timestamp
        stakes.timestamp = current_timestamp;
    
        // Ensure rewards are non-zero
        if rewards > 0 {
            // Transfer rewards (stETH) to the user
            let steth_address: Address = env.storage().instance().get(&DataKey::StETHAddress).unwrap();
            env.invoke_contract::<()>(
                &steth_address,
                &Symbol::new(&env, "transfer"),
                Vec::from_array(
                    &env,
                    [   
                        env.current_contract_address().into_val(&env),
                        user.into_val(&env),  
                        rewards.into_val(&env),
                    ]
                ),
            );
        } else {
            log!(&env, "No rewards to claim.");
        }
    
        // Update the stake in storage
        env.storage().instance().set(&DataKey::Stake(user.clone()), &stakes);
    
        rewards.into()  // Return the rewards amount
    }
    

    // New Verify Claim Function
    fn vclaim(env: Env, user: Address) -> i128 {
        let stakes: Stake = env.storage().instance().get(&DataKey::Stake(user.clone())).unwrap_or_default();

        log!(&env, "Stake Amount: {}", stakes.amount);
        // Assuming simple interest for rewards calculation:
        let reward_rate = 0.05; // 5% reward rate per unit of time
        let current_timestamp = env.ledger().timestamp();
        let staking_duration = current_timestamp - stakes.timestamp;
        log!(&env, "Staking Duration: {}", staking_duration , current_timestamp);
        let rewards: i128 = (stakes.amount as f64 * reward_rate * staking_duration as f64 / 1_000_000.0) as i128;
        rewards.into()
    }   
}
