use soroban_sdk::{contractclient, Address, Env};


/// Interface for StakingContract
#[contractclient(name = "ChrysalisClient")]

/// Trait defining the interface for a Staking Contract.
pub trait ChrysalisContractTrait {

    /*  *** Read-only functions: *** */

    fn initialize_contract(env: Env, staked_token_address: Address, steth_address: Address);

    /// Returns the current staked amount for the user.
    fn get_stake_amount(e: Env, user: Address) -> i128;

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
    fn stake_eth(e: Env, user: Address, amount: i128);

    /// Unstakes a specified amount using stETH, burns the stETH, and returns the original tokens.
    /// 
    /// # Arguments
    /// 
    /// * `e` - An instance of the `Env` struct.
    /// * `user` - The address of the user unstaking the tokens.
    /// * `amount` - The amount of stETH to use for unstaking.
    fn unstake_eth(e: Env, user: Address, amount: i128);

    fn claim(env: Env, user: Address , apy: i128)-> i128;

    fn vclaim(env: Env, user: Address , apy: i128)-> i128;
    
}
