use crate::{
    event_names, EscrowContract, EscrowContractClient, EscrowError, MultisigConfig,
    RENT_PERIOD_SECONDS,
};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    token, Address, BytesN, Env, FromVal, Symbol, TryFromVal, Val,
};

fn no_multisig(env: &Env) -> MultisigConfig {
    MultisigConfig {
        approvers: soroban_sdk::Vec::new(env),
        weights: soroban_sdk::Vec::new(env),
        threshold: 0,
    }
}

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    EscrowContractClient<'static>,
    u64,
) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let client_address = Address::generate(&env);
    let freelancer = Address::generate(&env);
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token_contract.address();
    token::StellarAssetClient::new(&env, &token_id).mint(&client_address, &1_100_i128);
    let escrow_id = client.create_escrow(
        &client_address,
        &freelancer,
        &token_id,
        &1_000_i128,
        &BytesN::from_array(&env, &[42; 32]),
        &None,
        &None,
        &None,
        &None,
        &no_multisig(&env),
        &None,
    );
    (
        env,
        contract_id,
        client_address,
        token_id,
        client,
        escrow_id,
    )
}

#[test]
fn rent_top_up_rejects_an_authenticated_non_client() {
    let (env, _, _, _, client, escrow_id) = setup();
    let outsider = Address::generate(&env);

    assert_eq!(
        client.try_top_up_rent(&outsider, &escrow_id, &1),
        Err(Ok(EscrowError::E5))
    );
}

#[test]
fn zero_period_top_up_is_a_safe_no_op() {
    let (_, _, client_address, _, client, escrow_id) = setup();
    assert_eq!(client.top_up_rent(&client_address, &escrow_id, &0), 0);
}

#[test]
fn rent_collection_event_contains_indexing_context() {
    let (env, contract_id, _, token_id, client, escrow_id) = setup();
    env.ledger()
        .with_mut(|ledger| ledger.timestamp += RENT_PERIOD_SECONDS);

    client.collect_rent(&escrow_id);

    let (_, topics, data) = env
        .events()
        .all()
        .iter()
        .find(|event| {
            event.0 == contract_id
                && Symbol::try_from_val(&env, &event.1.get(0).unwrap()).unwrap()
                    == event_names::RENT_COLLECTED
        })
        .expect("rent collection event missing");
    let indexed_escrow_id: u64 = FromVal::from_val(&env, &topics.get(1).unwrap());
    let (collected, remaining, expires_at, indexed_token): (i128, i128, u64, Address) =
        FromVal::from_val(&env, &data);
    assert_eq!(indexed_escrow_id, escrow_id);
    assert_eq!(collected, 1);
    assert!(remaining > 0);
    assert!(expires_at > env.ledger().timestamp());
    assert_eq!(indexed_token, token_id);
}

#[test]
fn failed_rent_lookup_emits_no_reclamation_event() {
    let (env, contract_id, _, _, client, _) = setup();
    assert_eq!(client.try_collect_rent(&999), Err(Ok(EscrowError::E8)));
    assert!(!env.events().all().iter().any(|event| {
        event.0 == contract_id
            && event
                .1
                .get(0)
                .and_then(|value| Symbol::try_from_val(&env, &value).ok())
                == Some(event_names::RENT_COLLECTED)
    }));
}
