use soroban_sdk::{contractclient, contractspecfn, Address, Env, BytesN};

pub struct Spec;

/// Interface for StakingContract
#[contractspecfn(name = "Spec", export = false)]
#[contractclient(name = "StakingContractClient")]

/// Trait defining the interface for a Staking Contract.
pub trait StakingContractTrait {

    /*  *** Read-only functions: *** */

    /// Returns the current staked amount for the user.
    fn get_stake(e: Env, user: Address) -> i64;

    /// Returns the current balance of stETH for the user.
    // fn get_steth_balance(e: Env, user: Address) -> i64;

    /*  *** State-Changing Functions: *** */

    /// Stakes a specified amount of the custom token, mints stETH, and locks it for the user.
    /// 
    /// # Arguments
    /// 
    /// * `e` - An instance of the `Env` struct.
    /// * `user` - The address of the user staking the tokens.
    /// * `amount` - The amount of the custom token to stake.
    fn stake(e: Env, user: Address, amount: i64);

    /// Unstakes a specified amount using stETH, burns the stETH, and returns the original tokens.
    /// 
    /// # Arguments
    /// 
    /// * `e` - An instance of the `Env` struct.
    /// * `user` - The address of the user unstaking the tokens.
    /// * `amount` - The amount of stETH to use for unstaking.
    fn unstake(e: Env, user: Address, amount: i64);
}
