use soroban_sdk::{contracttype, Address, Env};

use crate::types::Split;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    NextSplitId,
    Split(u64),
    FeeBps,
    Treasury,
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}

pub fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

pub fn get_next_split_id(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextSplitId)
        .unwrap_or(1u64)
}

pub fn bump_next_split_id(env: &Env) {
    let next = get_next_split_id(env);
    env.storage()
        .instance()
        .set(&DataKey::NextSplitId, &(next + 1));
}

pub fn set_split(env: &Env, split: &Split) {
    env.storage()
        .persistent()
        .set(&DataKey::Split(split.split_id), split);
}

pub fn get_split(env: &Env, split_id: u64) -> Option<Split> {
    env.storage().persistent().get(&DataKey::Split(split_id))
}

pub fn set_fee_bps(env: &Env, fee_bps: u32) {
    env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
}

pub fn get_fee_bps(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::FeeBps)
        .unwrap_or(0u32)
}

pub fn set_treasury(env: &Env, treasury: &Address) {
    env.storage().instance().set(&DataKey::Treasury, treasury);
}

pub fn get_treasury(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Treasury)
}
