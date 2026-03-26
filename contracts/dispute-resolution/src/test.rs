use crate::errors::Error;
use crate::types::{DisputeResult, DisputeStatus};
#[cfg(test)]
use crate::{DisputeContract, DisputeContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env, String,
};

fn setup() -> (Env, DisputeContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, DisputeContract);
    let client = DisputeContractClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_raise_dispute() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let id = client.raise_dispute(
        &String::from_str(&env, "split_001"),
        &raiser,
        &String::from_str(&env, "Payment was incorrect"),
    );

    let dispute = client.get_dispute(&id);
    assert_eq!(dispute.status, DisputeStatus::Voting);
    assert_eq!(dispute.votes_for, 0);
    assert_eq!(dispute.votes_against, 0);
    assert_eq!(dispute.voting_ends_at, 1000 + 604_800);
}

#[test]
fn test_vote_for_dispute() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let voter = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_002"),
        &raiser,
        &String::from_str(&env, "Wrong amount"),
    );

    client.vote_on_dispute(&id, &voter, &true);

    let dispute = client.get_dispute(&id);
    assert_eq!(dispute.votes_for, 1);
    assert_eq!(dispute.votes_against, 0);
}

#[test]
fn test_vote_against_dispute() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let voter = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_003"),
        &raiser,
        &String::from_str(&env, "Unfair split"),
    );

    client.vote_on_dispute(&id, &voter, &false);

    let dispute = client.get_dispute(&id);
    assert_eq!(dispute.votes_for, 0);
    assert_eq!(dispute.votes_against, 1);
}

#[test]
fn test_double_vote_fails() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let voter = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_004"),
        &raiser,
        &String::from_str(&env, "Duplicate payment"),
    );

    client.vote_on_dispute(&id, &voter, &true);

    let res = client.try_vote_on_dispute(&id, &voter, &true);
    assert!(matches!(res, Err(Ok(Error::AlreadyVoted))));
}

#[test]
fn test_resolve_upheld() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let voter1 = soroban_sdk::Address::generate(&env);
    let voter2 = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_005"),
        &raiser,
        &String::from_str(&env, "Missing funds"),
    );

    client.vote_on_dispute(&id, &voter1, &true);
    client.vote_on_dispute(&id, &voter2, &true);

    // Advance past voting period
    env.ledger().with_mut(|l| l.timestamp = 1000 + 604_801);

    let result = client.resolve_dispute(&id);
    assert_eq!(result, DisputeResult::UpheldForRaiser);

    let dispute = client.get_dispute(&id);
    assert_eq!(dispute.status, DisputeStatus::Resolved);
}

#[test]
fn test_resolve_dismissed() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let voter1 = soroban_sdk::Address::generate(&env);
    let voter2 = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_006"),
        &raiser,
        &String::from_str(&env, "Wrong recipient"),
    );

    client.vote_on_dispute(&id, &voter1, &false);
    client.vote_on_dispute(&id, &voter2, &false);

    env.ledger().with_mut(|l| l.timestamp = 1000 + 604_801);

    let result = client.resolve_dispute(&id);
    assert_eq!(result, DisputeResult::DismissedForRaiser);
}

#[test]
fn test_resolve_tied() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let voter1 = soroban_sdk::Address::generate(&env);
    let voter2 = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_007"),
        &raiser,
        &String::from_str(&env, "Unclear terms"),
    );

    client.vote_on_dispute(&id, &voter1, &true);
    client.vote_on_dispute(&id, &voter2, &false);

    env.ledger().with_mut(|l| l.timestamp = 1000 + 604_801);

    let result = client.resolve_dispute(&id);
    assert_eq!(result, DisputeResult::Tied);
}

#[test]
fn test_resolve_before_voting_ends_fails() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_008"),
        &raiser,
        &String::from_str(&env, "Too early"),
    );

    // Try to resolve immediately
    let res = client.try_resolve_dispute(&id);
    assert!(matches!(res, Err(Ok(Error::VotingPeriodActive))));
}

#[test]
fn test_vote_after_period_fails() {
    let (env, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let raiser = soroban_sdk::Address::generate(&env);
    let voter = soroban_sdk::Address::generate(&env);

    let id = client.raise_dispute(
        &String::from_str(&env, "split_009"),
        &raiser,
        &String::from_str(&env, "Late vote"),
    );

    // Advance past voting period then try to vote
    env.ledger().with_mut(|l| l.timestamp = 1000 + 604_801);

    let res = client.try_vote_on_dispute(&id, &voter, &true);
    assert!(matches!(res, Err(Ok(Error::VotingPeriodEnded))));
}
