#![no_std]

mod errors;
mod storage;
mod types;

#[cfg(test)]
mod test;

use errors::Error;
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String};
use types::{DataKey, Dispute, DisputeResult, DisputeStatus};

const VOTING_PERIOD: u64 = 604_800; // 7 days in seconds

fn generate_dispute_id(env: &Env, split_id: &String) -> String {
    let mut input = Bytes::new(env);
    input.append(&split_id.to_bytes());
    let seq = env.ledger().sequence().to_be_bytes();
    input.append(&Bytes::from_slice(env, &seq));
    let hash = env.crypto().sha256(&input);
    let hash_bytes = &hash.to_array()[..8];
    let mut id_bytes = String::from_str(env, "dis_").to_bytes();
    id_bytes.append(&Bytes::from_slice(env, hash_bytes));
    String::from_bytes(env, &id_bytes)
}

#[contract]
pub struct DisputeContract;

#[contractimpl]
impl DisputeContract {
    /// Raise a new dispute against a split.
    pub fn raise_dispute(
        env: Env,
        split_id: String,
        raiser: Address,
        reason: String,
    ) -> Result<String, Error> {
        raiser.require_auth();

        let now = env.ledger().timestamp();
        let dispute_id = generate_dispute_id(&env, &split_id);

        if storage::has_dispute(&env, &dispute_id) {
            return Err(Error::AlreadyExists);
        }

        let dispute = Dispute {
            dispute_id: dispute_id.clone(),
            split_id,
            raiser,
            reason,
            status: DisputeStatus::Voting,
            votes_for: 0,
            votes_against: 0,
            voters: soroban_sdk::Vec::new(&env),
            created_at: now,
            voting_ends_at: now + VOTING_PERIOD,
            result: None,
        };

        storage::save_dispute(&env, &dispute);
        storage::add_to_list(&env, dispute_id.clone());

        Ok(dispute_id)
    }

    /// Cast a vote on an open dispute.
    pub fn vote_on_dispute(
        env: Env,
        dispute_id: String,
        voter: Address,
        support: bool, // true = support the dispute, false = dismiss it
    ) -> Result<(), Error> {
        voter.require_auth();

        let mut dispute = storage::get_dispute(&env, &dispute_id)?;

        // Must be in Voting status
        if dispute.status != DisputeStatus::Voting {
            return Err(Error::DisputeClosed);
        }

        let now = env.ledger().timestamp();

        // Voting window must still be open
        if now > dispute.voting_ends_at {
            return Err(Error::VotingPeriodEnded);
        }

        // Each address can only vote once
        if storage::has_voted(&env, &dispute_id, &voter) {
            return Err(Error::AlreadyVoted);
        }

        // Record the vote
        if support {
            dispute.votes_for += 1;
        } else {
            dispute.votes_against += 1;
        }

        dispute.voters.push_back(voter.clone());
        storage::record_vote(&env, &dispute_id, &voter);
        storage::save_dispute(&env, &dispute);

        Ok(())
    }

    /// Resolve a dispute after voting period ends.
    pub fn resolve_dispute(env: Env, dispute_id: String) -> Result<DisputeResult, Error> {
        let mut dispute = storage::get_dispute(&env, &dispute_id)?;

        if dispute.status != DisputeStatus::Voting {
            return Err(Error::DisputeClosed);
        }

        let now = env.ledger().timestamp();

        // Voting period must have ended
        if now <= dispute.voting_ends_at {
            return Err(Error::VotingPeriodActive);
        }

        // Determine result based on votes
        let result = if dispute.votes_for > dispute.votes_against {
            DisputeResult::UpheldForRaiser
        } else if dispute.votes_against > dispute.votes_for {
            DisputeResult::DismissedForRaiser
        } else {
            DisputeResult::Tied
        };

        dispute.status = DisputeStatus::Resolved;
        dispute.result = Some(result.clone());

        storage::save_dispute(&env, &dispute);

        // TODO: trigger payout logic based on result
        // if result == DisputeResult::UpheldForRaiser {
        //     split_client.reverse_split(&dispute.split_id);
        // }

        Ok(result)
    }

    /// Get a dispute record.
    pub fn get_dispute(env: Env, dispute_id: String) -> Result<Dispute, Error> {
        storage::get_dispute(&env, &dispute_id)
    }

    /// Get all dispute IDs.
    pub fn get_all_disputes(env: Env) -> soroban_sdk::Vec<String> {
        storage::get_list(&env)
    }
}
