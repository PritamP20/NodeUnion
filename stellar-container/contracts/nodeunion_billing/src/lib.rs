#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

// Data structures
#[contracttype]
pub struct BillingConfig {
    pub authority: Address,
    pub token: Address,
    pub treasury: Address,
    pub rate_per_unit: u128,
}

#[contracttype]
pub enum DataKey {
    Config,
    Escrow(u64),
}

#[contracttype]
#[derive(Copy, Clone)]
pub enum EscrowStatus {
    Open,
    Closed,
}

#[contracttype]
pub struct JobEscrow {
    pub job_id: u64,
    pub user: Address,
    pub provider_wallet: Address,
    pub max_units: u128,
    pub used_units: u128,
    pub deposit_amount: u128,
    pub spent_amount: u128,
    pub status: EscrowStatus,
}

// Contract
#[contract]
pub struct BillingContract;

#[contractimpl]
impl BillingContract {
    pub fn initialize_config(
        env: Env,
        authority: Address,
        token: Address,
        treasury: Address,
        rate_per_unit: u128,
    ) {
        authority.require_auth();

        let config = BillingConfig {
            authority,
            token,
            treasury,
            rate_per_unit,
        };

        env.storage().persistent().set(&DataKey::Config, &config);
    }

    pub fn get_config(env: Env) -> BillingConfig {
        let config: BillingConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Config)
            .expect("Config not found");
        config
    }

    pub fn open_escrow(
        env: Env,
        job_id: u64,
        max_units: u128,
        deposit_amount: u128,
        provider_wallet: Address,
    ) {
        if deposit_amount == 0 {
            panic!("Invalid amount");
        }

        let caller = env.current_contract_address();

        let escrow = JobEscrow {
            job_id,
            user: caller,
            provider_wallet,
            max_units,
            used_units: 0,
            deposit_amount,
            spent_amount: 0,
            status: EscrowStatus::Open,
        };

        let key = get_escrow_key(job_id);
        env.storage().persistent().set(&key, &escrow);
    }

    pub fn get_escrow(env: Env, job_id: u64) -> JobEscrow {
        let key = get_escrow_key(job_id);
        let escrow: JobEscrow = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Escrow not found");
        escrow
    }

    pub fn record_usage(env: Env, job_id: u64, units: u128) {
        let config = Self::get_config(env.clone());
        config.authority.require_auth();

        if units == 0 {
            panic!("Invalid units");
        }

        let key = get_escrow_key(job_id);
        let mut escrow = Self::get_escrow(env.clone(), job_id);

        if !matches!(escrow.status, EscrowStatus::Open) {
            panic!("Escrow closed");
        }

        let new_used = escrow.used_units.saturating_add(units);
        if new_used > escrow.max_units {
            panic!("Usage exceeds max limit");
        }

        let amount = units.saturating_mul(config.rate_per_unit);
        let new_spent = escrow.spent_amount.saturating_add(amount);

        if new_spent > escrow.deposit_amount {
            panic!("Insufficient escrow");
        }

        escrow.used_units = new_used;
        escrow.spent_amount = new_spent;
        env.storage().persistent().set(&key, &escrow);
    }

    pub fn close_escrow(env: Env, job_id: u64) {
        let config = Self::get_config(env.clone());
        config.authority.require_auth();

        let key = get_escrow_key(job_id);
        let mut escrow = Self::get_escrow(env.clone(), job_id);

        if !matches!(escrow.status, EscrowStatus::Open) {
            panic!("Escrow already closed");
        }

        escrow.status = EscrowStatus::Closed;
        env.storage().persistent().set(&key, &escrow);
    }
}

fn get_escrow_key(job_id: u64) -> DataKey {
    DataKey::Escrow(job_id)
}
