#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};

mod errors;
mod events;
mod fees;
mod storage;
mod test;
mod types;

pub use crate::errors::Error;
pub use crate::types::{Split, SplitStatus};

#[contract]
pub struct SplitEscrowContract;

#[contractimpl]
impl SplitEscrowContract {
    pub fn initialize(env: Env, admin: Address, token_address: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token_address);
        storage::set_fee_bps(&env, 0u32);
        events::emit_initialized(&env, &admin);
        Ok(())
    }

    pub fn create_split(
        env: Env,
        creator: Address,
        description: String,
        total_amount: i128,
    ) -> Result<u64, Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }
        creator.require_auth();
        if total_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let split_id = storage::get_next_split_id(&env);
        storage::bump_next_split_id(&env);

        let split = Split {
            split_id,
            creator,
            description,
            total_amount,
            deposited_amount: 0,
            status: SplitStatus::Pending,
        };
        storage::set_split(&env, &split);
        events::emit_split_created(&env, &split);
        Ok(split_id)
    }

    pub fn deposit(
        env: Env,
        split_id: u64,
        participant: Address,
        amount: i128,
    ) -> Result<(), Error> {
        participant.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut split = storage::get_split(&env, split_id).ok_or(Error::SplitNotFound)?;
        if split.status != SplitStatus::Pending {
            return Err(Error::SplitNotPending);
        }
        if split.deposited_amount + amount > split.total_amount {
            return Err(Error::InvalidAmount);
        }

        let token_address = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&participant, &env.current_contract_address(), &amount);

        split.deposited_amount += amount;
        if split.deposited_amount == split.total_amount {
            split.status = SplitStatus::Ready;
        }
        storage::set_split(&env, &split);
        events::emit_deposit(&env, split_id, &participant, amount);
        Ok(())
    }

    pub fn release_funds(env: Env, split_id: u64) -> Result<(), Error> {
        let mut split = storage::get_split(&env, split_id).ok_or(Error::SplitNotFound)?;
        if split.status != SplitStatus::Ready {
            return Err(Error::SplitNotReady);
        }

        let total = split.deposited_amount;
        let fee_amount = fees::collect_fee(&env, total)?;
        let creator_amount = total - fee_amount;

        let token_address = storage::get_token(&env);
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &split.creator,
            &creator_amount,
        );

        split.status = SplitStatus::Released;
        storage::set_split(&env, &split);
        events::emit_released(&env, split_id, creator_amount);
        Ok(())
    }

    pub fn set_fee(env: Env, fee_bps: u32) -> Result<(), Error> {
        fees::set_fee(&env, fee_bps)
    }

    pub fn set_treasury(env: Env, address: Address) -> Result<(), Error> {
        fees::set_treasury(&env, &address)
    }

    pub fn get_split(env: Env, split_id: u64) -> Result<Split, Error> {
        storage::get_split(&env, split_id).ok_or(Error::SplitNotFound)
    }
}
