#![no_std]

mod contract_interface;

use crate::contract_interface::StakingContractClient;
use soroban_sdk::{contractimpl, contracttype, symbol, Address, BytesN, Env, IntoVal, Symbol};
use contract_interface::StakingContractTrait;

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

pub struct StakingContract;

#[contracttype]
enum DataKey {
    Stake(Address),
    StakedTokenAddress,
    StETHAddress,
}

#[contractimpl]
impl StakingContractTrait for StakingContract {
    fn initialize(env: Env, staked_token_address: Address, steth_address: Address) {
        env.storage().set(DataKey::StakedTokenAddress, staked_token_address);
        env.storage().set(DataKey::StETHAddress, steth_address);
    }

    fn stake(env: Env, user: Address, amount: i64) {
        let staked_token_address: Address = env.storage().get_unchecked(DataKey::StakedTokenAddress).unwrap();
        let steth_address: Address = env.storage().get_unchecked(DataKey::StETHAddress).unwrap();

        let balance: i64 = env.invoke_contract(
            &staked_token_address,
            &symbol!("balance_of"),
            (&user,).into_val(&env),
        );

        assert!(balance >= amount, "Insufficient balance");

        env.invoke_contract(
            &staked_token_address,
            &symbol!("transfer"),
            (&user, &env.current_contract_address(), &amount).into_val(&env),
        );

        env.invoke_contract(
            &steth_address,
            &symbol!("mint"),
            (&env.current_contract_address(), &user, &amount).into_val(&env),
        );

        let mut stakes = env.storage().get::<Address, Stake>(user.clone()).unwrap_or_default();
        stakes.amount += amount;
        stakes.timestamp = env.block().timestamp();
        env.storage().set(DataKey::Stake(user), stakes);
    }

    fn unstake(env: Env, user: Address, amount: i64) {
        let staked_token_address: Address = env.storage().get_unchecked(DataKey::StakedTokenAddress).unwrap();
        let steth_address: Address = env.storage().get_unchecked(DataKey::StETHAddress).unwrap();

        env.invoke_contract(
            &steth_address,
            &symbol!("burn"),
            (&user, &amount).into_val(&env),
        );

        env.invoke_contract(
            &staked_token_address,
            &symbol!("transfer"),
            (&env.current_contract_address(), &user, &amount).into_val(&env),
        );

        let mut stakes = env.storage().get::<Address, Stake>(user.clone()).unwrap_or_default();
        stakes.amount -= amount;
        env.storage().set(DataKey::Stake(user), stakes);
    }

    fn get_stake(env: Env, user: Address) -> i64 {
        let stakes = env.storage().get::<Address, Stake>(user).unwrap_or_default();
        stakes.amount
    }
}
