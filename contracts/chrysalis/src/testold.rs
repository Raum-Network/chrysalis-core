// #![cfg(test)]
// extern crate std;


// use core::fmt::Display;

// use soroban_sdk::{deploy::Deployer, testutils::{storage::Instance, Address}, Env, IntoVal, Symbol};
// use crate::{ ChrysalisContract, ChrysalisContractClient, DataKey , Stake};

// fn setup<'a>(env: &Env) -> ChrysalisContractClient<'a> {
//     // Create mock addresses for testing
//     // let staked_token_address = soroban_sdk::Address::generate(env);
//     // let steth_address = soroban_sdk::Address::generate(env);
//     // let user_address = soroban_sdk::Address::generate(env);

//     // Register and deploy the contract in the environment
//     let contract_id = env.register_contract(None, ChrysalisContract);

//     // Create the client to interact with the contract
//     let client: ChrysalisContractClient<'_> = ChrysalisContractClient::new(&env, &contract_id);

//     // Initialize the contract with mock addresses
//     client
// }


// #[test]
// fn test_initialize_contract() {
//     let env = Env::default();
//     env.mock_all_auths();
//     let staked_token_address = soroban_sdk::Address::generate(&env);
//     let steth_address = soroban_sdk::Address::generate(&env);
//     let _user_address = soroban_sdk::Address::generate(&env);
    
//     let contract_id = env.register_contract(None, ChrysalisContract);

//     // Create the client to interact with the contract
//     let test: ChrysalisContractClient<'_> = ChrysalisContractClient::new(&env, &contract_id);
    
//     test.initialize_contract(&staked_token_address, &steth_address);
//     // panic!("{:?}",test.vclaim(&_user_address.clone()));
//     // panic!("{:?}" , env.storage().instance().all() )
//     assert_eq!(env.storage().instance().get::<DataKey, soroban_sdk::Address>(&DataKey::StakedTokenAddress).unwrap(), staked_token_address);
//     // assert_eq!(env.storage().instance().get::<DataKey, soroban_sdk::Address>(&DataKey::StETHAddress).unwrap(), steth_address);
// }

// #[test]
// fn test_stake_eth() {
//     let env = Env::default();
//     env.mock_all_auths();
//     let user = soroban_sdk::Address::generate(&env);
//     let staked_token_address = soroban_sdk::Address::generate(&env);
//     let steth_address = soroban_sdk::Address::generate(&env);
//     let amount: i128 = 100;

//     // Mock the storage and contract calls
//     env.storage().instance().set(&DataKey::StakedTokenAddress, &staked_token_address);
//     env.storage().instance().set(&DataKey::StETHAddress, &steth_address);
//     env.storage().instance().set(&user, &Stake { amount: 10, timestamp: 0 });

//     // Mock the balance_of contract call to return a sufficient balance
//     let balance: i128 = env.invoke_contract(
//         &staked_token_address,
//         &Symbol::new(&env.clone(), "balance_of"),
//         (&user,).into_val(&env),
//     );

//     let contract_id = env.register_contract(None, ChrysalisContract);

//     // Create the client to interact with the contract
//     let test: ChrysalisContractClient<'_> = ChrysalisContractClient::new(&env, &contract_id);
    
//     test.initialize_contract(&staked_token_address, &steth_address);
//     // Call the stake_eth function
//     test.stake_eth( &user.clone(), &amount);

//     // Check the updated stake
//     let stakes: Stake = env.storage().instance().get(&user).unwrap();
//     assert_eq!(stakes.amount, amount);
//     assert_eq!(stakes.timestamp, env.ledger().timestamp());
// }


// // #[test]
// // fn test_unstake_eth() {
// //     let env = Env::default();
// //     let (contract_address, staked_token_address, steth_address) = setup(&env);
// //     let user_address = Address::from_string("user_address");

// //     // Setup initial stake
// //     let initial_stake = Stake { amount: 1000, timestamp: env.ledger().timestamp() };
// //     env.storage().instance().set(&DataKey::Stake(user_address.clone()), &initial_stake);

// //     // Call unstake_eth
// //     ChrysalisContract::unstake_eth(&env, user_address.clone(), 500);

// //     // Verify unstaking
// //     let stake = env.storage().instance().get(&DataKey::Stake(user_address)).unwrap();
// //     assert_eq!(stake.amount, 500);
// // }

// // #[test]
// // fn test_claim() {
// //     let env = Env::default();
// //     let (contract_address, staked_token_address, steth_address) = setup(&env);
// //     let user_address = Address::from_string("user_address");

// //     // Setup initial stake
// //     let initial_stake = Stake { amount: 1000, timestamp: env.ledger().timestamp() - 1_000_000 };
// //     env.storage().instance().set(&DataKey::Stake(user_address.clone()), &initial_stake);

// //     // Simulate minting rewards
// //     let expected_rewards = (1000.0 * 0.05 * 1_000_000.0 / 1_000_000.0) as i64;
// //     env.mock_contract()
// //         .expect("mint")
// //         .with_args(&contract_address, &user_address, &expected_rewards)
// //         .returns(());

// //     // Call claim
// //     let rewards = ChrysalisContract::claim(&env, user_address.clone());
    
// //     // Verify rewards and staking update
// //     assert_eq!(rewards, expected_rewards);
// //     let stake = env.storage().instance().get(&DataKey::Stake(user_address)).unwrap();
// //     assert_eq!(stake.timestamp, env.ledger().timestamp());
// // }

// // #[test]
// // fn test_vclaim() {
// //     let env = Env::default();
// //     let (contract_address, staked_token_address, steth_address) = setup(&env);
// //     let user_address = Address::from_string("user_address");

// //     // Setup initial stake
// //     let initial_stake = Stake { amount: 1000, timestamp: env.ledger().timestamp() - 1_000_000 };
// //     env.storage().instance().set(&DataKey::Stake(user_address.clone()), &initial_stake);

// //     // Calculate expected rewards
// //     let expected_rewards = (1000.0 * 0.05 * 1_000_000.0 / 1_000_000.0) as i64;

// //     // Call vclaim
// //     let rewards = ChrysalisContract::vclaim(&env, user_address.clone());

// //     // Verify rewards
// //     assert_eq!(rewards, expected_rewards);
// // }
