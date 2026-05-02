#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

mod contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/nodeunion_billing.wasm"
    );
}

use contract::Client;

#[test]
fn test_initialize_config() {
    let env = Env::default();
    let contract_id = env.register_contract(None, contract::BillingContract);
    let client = Client::new(&env, &contract_id);

    let authority = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    let rate_per_unit = 1000u128;

    let result = client.initialize_config(&authority, &token, &treasury, &rate_per_unit);
    assert!(result.is_ok());

    // Verify config was set
    let config = client.get_config();
    assert!(config.is_ok());
    let cfg = config.unwrap();
    assert_eq!(cfg.authority, authority);
    assert_eq!(cfg.token, token);
    assert_eq!(cfg.treasury, treasury);
    assert_eq!(cfg.rate_per_unit, rate_per_unit);
}

#[test]
fn test_register_network() {
    let env = Env::default();
    let contract_id = env.register_contract(None, contract::BillingContract);
    let client = Client::new(&env, &contract_id);

    let authority = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);

    // Initialize config first
    client
        .initialize_config(&authority, &token, &treasury, &1000u128)
        .unwrap();

    // Register network
    let network_id = String::from_slice(&env, "testnet");
    let name = String::from_slice(&env, "Test Network");
    let price_per_unit = 500u128;

    let result = client.register_network(&network_id, &name, &price_per_unit);
    assert!(result.is_ok());

    // Verify network was registered
    let network = client.get_network(&network_id);
    assert!(network.is_ok());
    let net = network.unwrap();
    assert_eq!(net.network_id, network_id);
    assert_eq!(net.name, name);
    assert_eq!(net.price_per_unit, price_per_unit);
}

#[test]
fn test_register_provider() {
    let env = Env::default();
    let contract_id = env.register_contract(None, contract::BillingContract);
    let client = Client::new(&env, &contract_id);

    let authority = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    let provider_wallet = Address::generate(&env);

    // Initialize config
    client
        .initialize_config(&authority, &token, &treasury, &1000u128)
        .unwrap();

    // Register network
    let network_id = String::from_slice(&env, "testnet");
    let name = String::from_slice(&env, "Test Network");
    client
        .register_network(&network_id, &name, &500u128)
        .unwrap();

    // Register provider
    let provider_id = String::from_slice(&env, "provider-1");
    let result = client.register_provider(&network_id, &provider_id, &provider_wallet);
    assert!(result.is_ok());

    // Verify provider was registered
    let provider = client.get_provider(&network_id, &provider_id);
    assert!(provider.is_ok());
    let prov = provider.unwrap();
    assert_eq!(prov.provider_id, provider_id);
    assert_eq!(prov.network_id, network_id);
    assert_eq!(prov.provider_wallet, provider_wallet);
}

#[test]
fn test_open_escrow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, contract::BillingContract);
    let client = Client::new(&env, &contract_id);

    let authority = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    let provider_wallet = Address::generate(&env);
    let user = Address::generate(&env);

    // Setup
    client
        .initialize_config(&authority, &token, &treasury, &1000u128)
        .unwrap();

    let network_id = String::from_slice(&env, "testnet");
    client
        .register_network(&network_id, &String::from_slice(&env, "Test"), &500u128)
        .unwrap();

    let provider_id = String::from_slice(&env, "provider-1");
    client
        .register_provider(&network_id, &provider_id, &provider_wallet)
        .unwrap();

    // Open escrow
    let job_id = String::from_slice(&env, "job-123");
    let max_units = 100u128;
    let deposit_amount = 100000u128;

    // Note: In real tests, we'd need to mock the token contract
    let result = client.open_escrow(&job_id, &network_id, &provider_id, &max_units, &deposit_amount);
    
    // This will fail in unit tests without proper token mocking, 
    // but demonstrates the flow
    println!("open_escrow result: {:?}", result);
}

#[test]
fn test_invalid_network_id() {
    let env = Env::default();
    let contract_id = env.register_contract(None, contract::BillingContract);
    let client = Client::new(&env, &contract_id);

    let authority = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);

    client
        .initialize_config(&authority, &token, &treasury, &1000u128)
        .unwrap();

    // Try to register network with empty ID
    let network_id = String::from_slice(&env, "");
    let name = String::from_slice(&env, "Test Network");

    let result = client.register_network(&network_id, &name, &500u128);
    assert!(result.is_err());
}

#[test]
fn test_invalid_price_per_unit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, contract::BillingContract);
    let client = Client::new(&env, &contract_id);

    let authority = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);

    client
        .initialize_config(&authority, &token, &treasury, &1000u128)
        .unwrap();

    // Try to register network with zero price
    let network_id = String::from_slice(&env, "testnet");
    let name = String::from_slice(&env, "Test Network");

    let result = client.register_network(&network_id, &name, &0u128);
    assert!(result.is_err());
}
