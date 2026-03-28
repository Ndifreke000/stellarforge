//! # StellarForge Contract Benchmarks
//!
//! Measures CPU instructions and memory bytes consumed by each contract's
//! core operations using Soroban's built-in budget tracking.
//!
//! Run with: `cargo run -p forge-benches`

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Symbol,
};

// Re-export generated clients from each contract crate
use forge_governor::{GovernorConfig, GovernorContractClient};
use forge_multisig::MultisigContractClient;
use forge_oracle::ForgeOracleClient;
use forge_stream::ForgeStreamClient;
use forge_vesting::ForgeVestingClient;

fn print_budget(label: &str, env: &Env) {
    let cpu = env.budget().cpu_instruction_cost();
    let mem = env.budget().memory_bytes_cost();
    println!("  {label:<35} cpu={cpu:>12} instructions   mem={mem:>10} bytes");
}

fn bench_vesting(env: &Env) {
    println!("\n[forge-vesting]");

    let contract_id = env.register_contract(None, forge_vesting::ForgeVesting);
    let client = ForgeVestingClient::new(env, &contract_id);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let beneficiary = Address::generate(env);
    let admin = Address::generate(env);
    soroban_sdk::token::StellarAssetClient::new(env, &token).mint(&contract_id, &1_000_000);

    env.budget().reset_default();
    client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
    print_budget("initialize()", env);

    env.ledger().with_mut(|l| l.timestamp = 500);
    env.budget().reset_default();
    client.claim();
    print_budget("claim() at 50%", env);

    env.ledger().with_mut(|l| l.timestamp = 1000);
    env.budget().reset_default();
    client.claim();
    print_budget("claim() at 100%", env);
}

fn bench_stream(env: &Env) {
    println!("\n[forge-stream]");

    let contract_id = env.register_contract(None, forge_stream::ForgeStream);
    let client = ForgeStreamClient::new(env, &contract_id);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let sender = Address::generate(env);
    let recipient = Address::generate(env);
    soroban_sdk::token::StellarAssetClient::new(env, &token).mint(&sender, &1_000_000);

    env.budget().reset_default();
    let stream_id = client.create_stream(&sender, &token, &recipient, &100, &10000);
    print_budget("create_stream()", env);

    env.ledger().with_mut(|l| l.timestamp += 5000);
    env.budget().reset_default();
    client.withdraw(&stream_id);
    print_budget("withdraw() at 50%", env);

    env.budget().reset_default();
    client.cancel_stream(&stream_id);
    print_budget("cancel_stream()", env);
}

fn bench_multisig(env: &Env) {
    println!("\n[forge-multisig]");

    let contract_id = env.register_contract(None, forge_multisig::MultisigContract);
    let client = MultisigContractClient::new(env, &contract_id);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let owner_a = Address::generate(env);
    let owner_b = Address::generate(env);
    let owner_c = Address::generate(env);
    let recipient = Address::generate(env);
    soroban_sdk::token::StellarAssetClient::new(env, &token).mint(&contract_id, &500_000);

    let owners = soroban_sdk::vec![env, owner_a.clone(), owner_b.clone(), owner_c.clone()];

    env.budget().reset_default();
    client.initialize(&owners, &2, &0);
    print_budget("initialize() 3-of-3, threshold=2", env);

    env.budget().reset_default();
    let proposal_id = client.propose(&owner_a, &recipient, &token, &100_000);
    print_budget("propose()", env);

    env.budget().reset_default();
    client.approve(&owner_b, &proposal_id);
    print_budget("approve() (reaches threshold)", env);

    env.budget().reset_default();
    client.execute(&owner_a, &proposal_id);
    print_budget("execute()", env);
}

fn bench_governor(env: &Env) {
    println!("\n[forge-governor]");

    let contract_id = env.register_contract(None, forge_governor::GovernorContract);
    let client = GovernorContractClient::new(env, &contract_id);
    let token_admin = Address::generate(env);
    let vote_token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let proposer = Address::generate(env);
    let voter = Address::generate(env);
    soroban_sdk::token::StellarAssetClient::new(env, &vote_token).mint(&voter, &1_000_000);

    let config = GovernorConfig {
        vote_token: vote_token.clone(),
        voting_period: 1000,
        quorum: 500_000,
        timelock_delay: 0,
    };

    env.budget().reset_default();
    client.initialize(&config);
    print_budget("initialize()", env);

    env.budget().reset_default();
    let proposal_id = client.propose(
        &proposer,
        &String::from_str(env, "Upgrade fee"),
        &String::from_str(env, "Raise protocol fee to 0.3%"),
    );
    print_budget("propose()", env);

    env.budget().reset_default();
    client.vote(&voter, &proposal_id, &true, &1_000_000);
    print_budget("vote()", env);

    env.ledger().with_mut(|l| l.timestamp += 1001);
    env.budget().reset_default();
    client.finalize(&proposal_id);
    print_budget("finalize()", env);
}

fn bench_oracle(env: &Env) {
    println!("\n[forge-oracle]");

    let contract_id = env.register_contract(None, forge_oracle::ForgeOracle);
    let client = ForgeOracleClient::new(env, &contract_id);
    let admin = Address::generate(env);

    env.budget().reset_default();
    client.initialize(&admin, &3600);
    print_budget("initialize()", env);

    let base = Symbol::new(env, "XLM");
    let quote = Symbol::new(env, "USDC");

    env.budget().reset_default();
    client.submit_price(&base, &quote, &10_000_000);
    print_budget("submit_price()", env);

    env.budget().reset_default();
    client.get_price(&base, &quote);
    print_budget("get_price()", env);

    env.budget().reset_default();
    client.get_all_prices();
    print_budget("get_all_prices() (1 pair)", env);
}

fn main() {
    let env = Env::default();
    env.mock_all_auths();

    println!("StellarForge Contract Benchmarks");
    println!("=================================");
    println!("Metric: Soroban budget (CPU instructions + memory bytes)");
    println!("Note:   Each measurement is reset before the call under test.");

    bench_vesting(&env);
    bench_stream(&env);
    bench_multisig(&env);
    bench_governor(&env);
    bench_oracle(&env);

    println!("\nDone.");
}
