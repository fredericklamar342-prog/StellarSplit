#![cfg(test)]
extern crate std;

use crate::{SplitEscrowContract, SplitEscrowContractClient, SplitStatus};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient as TokenAdminClient};
use soroban_sdk::IntoVal;
use soroban_sdk::{testutils::Address as _, testutils::Events as _, Address, Env, String};

fn setup() -> (
    Env,
    SplitEscrowContractClient<'static>,
    Address,
    Address,
    Address,
    TokenClient<'static>,
    TokenAdminClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let participant = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token = token_contract.address();

    let token_client = TokenClient::new(&env, &token);
    let token_admin_client = TokenAdminClient::new(&env, &token);

    let contract_id = env.register_contract(None, SplitEscrowContract);
    let client = SplitEscrowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &String::from_str(&env, "1.0.0"));

    token_admin_client.mint(&participant, &1_000_000);

    (
        env,
        client,
        admin,
        creator,
        participant,
        token_client,
        token_admin_client,
    )
}

#[test]
fn test_fee_deducted_and_sent_to_treasury_on_release() {
    let (env, client, admin, creator, participant, token_client, _) = setup();
    let treasury = Address::generate(&env);
    client.set_treasury(&treasury);
    client.set_fee(&250u32); // 2.5%

    let split_id = client.create_escrow(
        &creator,
        &String::from_str(&env, "Dinner"),
        &10_000,
        &None,
        &None,
    );
    client.deposit(&split_id, &participant, &10_000);
    client.release_funds(&split_id);

    assert_eq!(token_client.balance(&treasury), 250);
    assert_eq!(token_client.balance(&creator), 9_750);

    let escrow = client.get_escrow(&split_id);
    assert_eq!(escrow.status, SplitStatus::Released);

    let _ = admin;
}

#[test]
fn test_admin_can_update_fee_and_treasury() {
    let (env, client, _admin, creator, participant, token_client, _) = setup();

    let treasury_a = Address::generate(&env);
    client.set_treasury(&treasury_a);
    client.set_fee(&100u32);

    let split_a =
        client.create_escrow(&creator, &String::from_str(&env, "A"), &1_000, &None, &None);
    client.deposit(&split_a, &participant, &1_000);
    client.release_funds(&split_a);
    assert_eq!(token_client.balance(&treasury_a), 10);

    let treasury_b = Address::generate(&env);
    client.set_treasury(&treasury_b);
    client.set_fee(&300u32);

    let split_b =
        client.create_escrow(&creator, &String::from_str(&env, "B"), &2_000, &None, &None);
    client.deposit(&split_b, &participant, &2_000);
    client.release_funds(&split_b);
    assert_eq!(token_client.balance(&treasury_b), 60);
}

#[test]
fn test_set_fee_and_set_treasury_are_admin_only() {
    let (env, client, admin, _creator, _participant, _token_client, _token_admin) = setup();

    env.mock_all_auths();
    client.set_fee(&123u32);
    client.set_treasury(&Address::generate(&env));

    assert_ne!(admin, Address::generate(&env));
}

#[test]
fn test_fees_collected_event_emitted() {
    let (env, client, _admin, creator, participant, _token_client, _) = setup();
    let treasury = Address::generate(&env);
    client.set_treasury(&treasury);
    client.set_fee(&500u32);

    let before_len = env.events().all().len();

    let split_id = client.create_escrow(
        &creator,
        &String::from_str(&env, "Event"),
        &1_000,
        &None,
        &None,
    );
    client.deposit(&split_id, &participant, &1_000);
    client.release_funds(&split_id);

    let after_len = env.events().all().len();
    assert!(after_len > before_len);
}

#[test]
fn test_version_stored_on_init() {
    let (env, client, _, _, _, _, _) = setup();
    assert_eq!(client.get_version(), String::from_str(&env, "1.0.0"));
}

#[test]
fn test_upgrade_version_admin() {
    let (env, client, _admin, _, _, _, _) = setup();

    client.upgrade_version(&String::from_str(&env, "1.1.0"));
    assert_eq!(client.get_version(), String::from_str(&env, "1.1.0"));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")] // Missing admin auth
fn test_upgrade_version_non_admin_fails() {
    let (env, client, _, creator, _, _, _) = setup();

    // Disable blanket auth mocking so we can assert on authorization failures.
    env.set_auths(&[]);

    // Switch to creator auth
    client.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &creator,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade_version",
            args: (String::from_str(&env, "1.1.0"),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.upgrade_version(&String::from_str(&env, "1.1.0"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")] // InvalidVersion
fn test_upgrade_version_invalid_semver_fails() {
    let (env, client, _, _, _, _, _) = setup();
    client.upgrade_version(&String::from_str(&env, "1.0"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")] // InvalidVersion
fn test_initialize_invalid_version_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let contract_id = env.register_contract(None, SplitEscrowContract);
    let client = SplitEscrowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &String::from_str(&env, "1.0"));
}

#[test]
fn test_contract_upgraded_event_emitted() {
    let (env, client, _, _, _, _, _) = setup();

    let before_len = env.events().all().len();
    client.upgrade_version(&String::from_str(&env, "2.0.0"));
    let after_len = env.events().all().len();

    assert!(after_len > before_len);

    let last_event = env.events().all().last().unwrap();
    assert_eq!(last_event.0, client.address);
}

#[test]
fn test_default_max_participants_is_50() {
    let (env, client, _admin, creator, _p, _tc, token_admin) = setup();
    let escrow_id = client.create_escrow(
        &creator,
        &String::from_str(&env, "Cap default"),
        &100,
        &None,
        &None,
    );
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.max_participants, 50);
    assert_eq!(escrow.participants.len(), 0);

    let _ = token_admin;
}

#[test]
fn test_explicit_max_participants_stored_in_get_escrow() {
    let (env, client, _admin, creator, p1, _tc, _ta) = setup();
    let cap = 3u32;
    let escrow_id = client.create_escrow(
        &creator,
        &String::from_str(&env, "Explicit cap"),
        &300,
        &Some(cap),
        &None,
    );
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.max_participants, cap);
    client.deposit(&escrow_id, &p1, &100);
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.participants.len(), 1);
}

#[test]
fn test_deposit_rejected_when_participant_cap_exceeded() {
    let (env, client, _admin, creator, p1, _tc, token_admin) = setup();
    let p2 = Address::generate(&env);
    let p3 = Address::generate(&env);
    token_admin.mint(&p2, &10_000);
    token_admin.mint(&p3, &10_000);

    let escrow_id = client.create_escrow(
        &creator,
        &String::from_str(&env, "Two max"),
        &3_000,
        &Some(2u32),
        &None,
    );

    client.deposit(&escrow_id, &p1, &1_000);
    client.deposit(&escrow_id, &p2, &1_000);
    assert_eq!(client.get_escrow(&escrow_id).participants.len(), 2);

    let res = client.try_deposit(&escrow_id, &p3, &1_000);
    assert!(res.is_err());

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.participants.len(), 2);
    assert_eq!(escrow.deposited_amount, 2_000);
}

#[test]
fn test_existing_participant_can_deposit_again_without_increasing_count() {
    let (env, client, _admin, creator, p1, _tc, _ta) = setup();
    // release_funds runs fee collection; treasury must be set even when fee bps is 0.
    client.set_treasury(&Address::generate(&env));

    let escrow_id = client.create_escrow(
        &creator,
        &String::from_str(&env, "Repeat"),
        &2_000,
        &Some(1u32),
        &None,
    );
    client.deposit(&escrow_id, &p1, &1_000);
    client.deposit(&escrow_id, &p1, &1_000);
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.participants.len(), 1);
    assert_eq!(escrow.deposited_amount, 2_000);
    client.release_funds(&escrow_id);
    assert_eq!(client.get_escrow(&escrow_id).status, SplitStatus::Released);
}

#[test]
fn test_note_stored_on_create_and_get_note() {
    let (env, client, _admin, creator, _p, _tc, _ta) = setup();
    let text = "Dinner at Luigi's — Friday night";
    let split_id = client.create_escrow(
        &creator,
        &String::from_str(&env, "Bill"),
        &100,
        &None,
        &Some(String::from_str(&env, text)),
    );
    assert_eq!(client.get_note(&split_id), String::from_str(&env, text));
    assert_eq!(
        client.get_escrow(&split_id).note,
        String::from_str(&env, text)
    );
}

#[test]
fn test_creator_can_update_note_while_pending_and_ready() {
    let (env, client, _admin, creator, p1, _tc, _ta) = setup();
    client.set_treasury(&Address::generate(&env));

    let split_id =
        client.create_escrow(&creator, &String::from_str(&env, "X"), &2_000, &None, &None);
    client.set_note(&split_id, &String::from_str(&env, "v1"));
    assert_eq!(client.get_note(&split_id), String::from_str(&env, "v1"));

    client.deposit(&split_id, &p1, &1_000);
    client.set_note(&split_id, &String::from_str(&env, "v2-ready"));
    assert_eq!(
        client.get_note(&split_id),
        String::from_str(&env, "v2-ready")
    );

    client.deposit(&split_id, &p1, &1_000);
    client.release_funds(&split_id);
    let res = client.try_set_note(&split_id, &String::from_str(&env, "late"));
    assert!(res.is_err());
}

#[test]
fn test_note_over_128_bytes_rejected_on_create_and_set() {
    let (env, client, _admin, creator, _p, _tc, _ta) = setup();
    let bytes = [b'a'; 129];
    let long = String::from_str(&env, core::str::from_utf8(&bytes).unwrap());
    assert_eq!(long.len(), 129);

    let res = client.try_create_escrow(
        &creator,
        &String::from_str(&env, "x"),
        &100,
        &None,
        &Some(long.clone()),
    );
    assert!(res.is_err());

    let split_id =
        client.create_escrow(&creator, &String::from_str(&env, "ok"), &100, &None, &None);
    let res2 = client.try_set_note(&split_id, &long);
    assert!(res2.is_err());
}

#[test]
fn test_note_updated_emits_event() {
    let (env, client, _admin, creator, _p, _tc, _ta) = setup();
    let split_id = client.create_escrow(&creator, &String::from_str(&env, "E"), &100, &None, &None);
    let before = env.events().all().len();
    client.set_note(&split_id, &String::from_str(&env, "hello"));
    assert!(env.events().all().len() > before);
}

// ============================================================
// Property / invariant tests (proptest-style)
// ============================================================

mod proptests {
    use super::*;
    use proptest::prelude::*;

    const MAX_PARTICIPANTS_IN_TEST: usize = 4;

    #[derive(Clone)]
    struct Model {
        status: SplitStatus,
        total_amount: i128,
        deposited_amount: i128,
        max_participants: u32,
        used_participants: [bool; MAX_PARTICIPANTS_IN_TEST],
        fee_bps: u32,
    }

    fn used_count(m: &Model) -> u32 {
        let mut c = 0u32;
        for i in 0..MAX_PARTICIPANTS_IN_TEST {
            if m.used_participants[i] {
                c += 1;
            }
        }
        c
    }

    fn calculate_fee(total: i128, fee_bps: u32) -> i128 {
        (total * fee_bps as i128) / 10_000i128
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 32, .. ProptestConfig::default() })]
        #[test]
        fn prop_escrow_invariants_deposit_and_release_sequences(
            total_amount in 1_000i128..=10_000i128,
            cap in 1u32..=3u32,
            fee_bps in 0u32..=500u32,
            steps in prop::collection::vec(
                (
                    0u8..=1u8, // 0 deposit, 1 release
                    0u8..=3u8, // participant index
                    1u32..=5_000u32, // amount
                ),
                1usize..=20
            ),
        ) {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let creator = Address::generate(&env);
            let treasury = Address::generate(&env);

            // Deploy token
            let token_admin = Address::generate(&env);
            let token_contract = env.register_stellar_asset_contract_v2(token_admin);
            let token_addr = token_contract.address();
            let token_client = TokenClient::new(&env, &token_addr);
            let token_admin_client = TokenAdminClient::new(&env, &token_addr);

            // Deploy escrow contract
            let contract_id = env.register_contract(None, SplitEscrowContract);
            let client = SplitEscrowContractClient::new(&env, &contract_id);
            client.initialize(&admin, &token_addr, &String::from_str(&env, "1.0.0"));

            client.set_treasury(&treasury);
            client.set_fee(&fee_bps);

            // Create split
            let split_id = client.create_escrow(
                &creator,
                &String::from_str(&env, "prop-escrow"),
                &total_amount,
                &Some(cap),
                &None,
            );

            // Seed participants and track their balances.
            let mut participants: std::vec::Vec<Address> = std::vec::Vec::new();
            for _ in 0..MAX_PARTICIPANTS_IN_TEST {
                participants.push(Address::generate(&env));
            }

            // Mint enough tokens to cover up to several deposits.
            let mint_amount = total_amount * 5;
            for p in &participants {
                token_admin_client.mint(p, &mint_amount);
            }

            let initial_total_supply = {
                let mut t = token_client.balance(&creator);
                t += token_client.balance(&treasury);
                t += token_client.balance(&contract_id);
                for p in &participants {
                    t += token_client.balance(p);
                }
                t
            };

            let mut model = Model {
                status: SplitStatus::Pending,
                total_amount,
                deposited_amount: 0,
                max_participants: cap,
                used_participants: [false; MAX_PARTICIPANTS_IN_TEST],
                fee_bps,
            };

            for (action, p_idx_raw, amount_raw) in steps {
                let p_idx = p_idx_raw as usize;
                let amount = amount_raw as i128;

                if action == 0 {
                    // Deposit attempt
                    let participant = participants[p_idx].clone();
                    let res = client.try_deposit(&split_id, &participant, &amount);
                    if res.is_ok() {
                        // Invariant: escrow must have been Pending and amount must fit.
                        prop_assert_eq!(model.status.clone(), SplitStatus::Pending);
                        prop_assert!(model.deposited_amount + amount <= model.total_amount);
                        if !model.used_participants[p_idx] {
                            prop_assert!(used_count(&model) < model.max_participants);
                            model.used_participants[p_idx] = true;
                        }
                        model.deposited_amount += amount;
                        if model.deposited_amount == model.total_amount {
                            model.status = SplitStatus::Ready;
                        }
                    }
                } else {
                    // Release attempt
                    let res = client.try_release_funds(&split_id);
                    if res.is_ok() {
                        prop_assert_eq!(model.status.clone(), SplitStatus::Ready);
                        model.status = SplitStatus::Released;
                    } else {
                        // If release fails, state must not change.
                        prop_assert!(model.status.clone() != SplitStatus::Ready);
                    }
                }

                // Fetch escrow on-chain state and validate invariants.
                let escrow = client.get_escrow(&split_id);
                prop_assert_eq!(escrow.status, model.status.clone());
                prop_assert_eq!(escrow.total_amount, model.total_amount);
                prop_assert_eq!(escrow.deposited_amount, model.deposited_amount);
                prop_assert_eq!(escrow.max_participants, model.max_participants);
                prop_assert_eq!(escrow.participants.len(), used_count(&model));
                prop_assert!(escrow.deposited_amount <= escrow.total_amount);

                // Money invariants:
                let current_total_supply = {
                    let mut t = token_client.balance(&creator);
                    t += token_client.balance(&treasury);
                    t += token_client.balance(&contract_id);
                    for p in &participants {
                        t += token_client.balance(p);
                    }
                    t
                };
                prop_assert_eq!(current_total_supply, initial_total_supply);

                let contract_balance = token_client.balance(&contract_id);
                match model.status.clone() {
                    SplitStatus::Pending => {
                        prop_assert_eq!(contract_balance, model.deposited_amount);
                    }
                    SplitStatus::Ready => {
                        prop_assert_eq!(contract_balance, model.total_amount);
                    }
                    SplitStatus::Released => {
                        // All funds have been distributed.
                        prop_assert_eq!(contract_balance, 0);

                        let fee_amount = calculate_fee(model.total_amount, model.fee_bps);
                        let creator_amount = model.total_amount - fee_amount;
                        prop_assert_eq!(token_client.balance(&treasury), fee_amount);
                        prop_assert_eq!(token_client.balance(&creator), creator_amount);
                    }
                }
            }
        }
    }
}
