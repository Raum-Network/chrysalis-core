#![no_std]

mod contract_interface;

use soroban_sdk::{contractimpl , contract, contracttype, Address, Env, IntoVal, Symbol};
use contract_interface::ChrysalisContractTrait;

#[derive(Clone, Debug)]
pub struct Stake {
    amount: i64,
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

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Stake(Address),
    StakedTokenAddress,
    StETHAddress,
}

#[contractimpl]
impl ChrysalisContractTrait for ChrysalisContract {
    
    fn initializeContract(env: Env, staked_token_address: Address, steth_address: Address) {
        env.storage().instance().set(&DataKey::StakedTokenAddress, &staked_token_address);
        env.storage().instance().set(&DataKey::StETHAddress, &steth_address);
    }

    fn stakeETH(env: Env, user: Address, amount: i64) {
        let staked_token_address: Address = env.storage().instance().get(&DataKey::StakedTokenAddress).unwrap();
        let steth_address: Address = env.storage().instance().get(&DataKey::StETHAddress).unwrap();

        let balance: i64 = env.invoke_contract(
            &staked_token_address,
            &Symbol::new(&env.clone(), "balance_of"),
            (&user,).into_val(&env),
        );

        assert!(balance >= amount, "Insufficient balance");

        env.invoke_contract(
            &staked_token_address,
            &Symbol::new(&env.clone(), "transfer"),
            (&user, &env.current_contract_address(), &amount).into_val(&env),
        );

        env.invoke_contract(
            &steth_address,
            &Symbol::new(&env.clone(), "mint"),
            (&env.current_contract_address(), &user, &amount).into_val(&env),
        );

        let mut stakes = env.storage().instance().get::<Address, Stake>(&user.clone()).unwrap_or_default();
        stakes.amount += amount;
        stakes.timestamp = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Stake(user), &stakes);
    }

    fn unstakeETH(env: Env, user: Address, amount: i64) {
        let staked_token_address: Address = env.storage().get_unchecked(DataKey::StakedTokenAddress).unwrap();
        let steth_address: Address = env.storage().get_unchecked(DataKey::StETHAddress).unwrap();

        env.invoke_contract(
            &steth_address,
            &Symbol::new(env.clone(), "burn"),
            (&user, &amount).into_val(&env),
        );

        env.invoke_contract(
            &staked_token_address,
            &Symbol::new(env.clone(), "transfer"),
            (&env.current_contract_address(), &user, &amount).into_val(&env),
        );

        let mut stakes = env.storage().get::<Address, Stake>(user.clone()).unwrap_or_default();
        stakes.amount -= amount;
        env.storage().set(DataKey::Stake(user), stakes);
    }

    fn get_stake_amount(env: Env, user: Address) -> i64 {
        let stakes = env.storage().get::<Address, Stake>(user).unwrap_or_default();
        stakes.amount
    }
}
