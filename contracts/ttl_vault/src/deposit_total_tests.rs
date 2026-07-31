#![cfg(test)]

extern crate alloc;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup_deposit_total_test() -> (Env, Address, Address, TtlVaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let admin = Address::generate(&env);

    let token_address = Address::generate(&env);
    let contract_id = env.register_contract(None, TtlVaultContract);
    let client: TtlVaultContractClient<'static> = unsafe { core::mem::transmute(TtlVaultContractClient::new(&env, &contract_id)) };

    client.initialize(&token_address, &admin);

    (env, owner, beneficiary, client)
}

#[test]
fn test_get_deposit_total_funded_vault() {
    let (env, owner, beneficiary, client) = setup_deposit_total_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &86400, &None);

    client.deposit(&vault_id, &owner, &500_000i128);
    let total = client.get_deposit_total(&vault_id);
    assert_eq!(total, Ok(500_000i128));
}

#[test]
fn test_get_deposit_total_empty_vault() {
    let (env, owner, beneficiary, client) = setup_deposit_total_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &86400, &None);

    let total = client.get_deposit_total(&vault_id);
    assert_eq!(total, Ok(0i128));
}

#[test]
fn test_get_deposit_total_non_existent_vault() {
    let (_env, _owner, _beneficiary, client) = setup_deposit_total_test();

    let result = client.try_get_deposit_total(&999_999u64);
    assert!(result.is_err());
    match result.unwrap_err().unwrap() {
        ContractError::VaultNotFound => {}
        e => panic!("Expected VaultNotFound, got {:?}", e),
    }
}
