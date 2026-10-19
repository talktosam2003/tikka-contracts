//! Config validation and init rejection branches (claim lockup, bounds, metadata hash).

use super::*;

#[test]
fn test_init_claim_lockup_seconds_at_bound_succeeds() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let mut config = base_config(&env, &payment_token);
    config.claim_lockup_seconds = Some(MAX_CLAIM_LOCKUP_SECONDS);

    client.init(&factory, &admin, &creator, &config);

    assert_eq!(client.get_raffle().claim_lockup_seconds, MAX_CLAIM_LOCKUP_SECONDS);
}

#[test]
fn test_init_claim_lockup_seconds_above_bound_rejected() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let mut config = base_config(&env, &payment_token);
    config.claim_lockup_seconds = Some(MAX_CLAIM_LOCKUP_SECONDS + 1);

    assert_eq!(
        client.try_init(&factory, &admin, &creator, &config),
        Err(Ok(Error::InvalidParameters))
    );
}



/// #485: with unique_winners, two buyers with multiple tickets each win at most one tier.

#[test]
fn test_init_claim_lockup_seconds_mid_range_succeeds() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let lockup_seconds = MAX_CLAIM_LOCKUP_SECONDS / 2;
    let mut config = base_config(&env, &payment_token);
    config.claim_lockup_seconds = Some(lockup_seconds);

    client.init(&factory, &admin, &creator, &config);

    assert_eq!(client.get_raffle().claim_lockup_seconds, lockup_seconds);
}

#[test]
fn init_accepts_min_ticket_price_and_rejects_below_it() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let prizes = soroban_sdk::vec![&env, 10000u32];

    let config = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "min ticket price"),
        5,
        MIN_TICKET_PRICE,
        MIN_TICKET_PRICE * 5,
        prizes.clone(),
    );
    client.init(&factory, &admin, &creator, &config);

    let invalid = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "min ticket price"),
        5,
        MIN_TICKET_PRICE - 1,
        MIN_TICKET_PRICE * 5,
        prizes,
    );
    assert_eq!(
        client.try_init(&factory, &admin, &creator, &invalid),
        Err(Ok(Error::InvalidParameters))
    );
}

#[test]
fn init_accepts_max_prize_amount_and_rejects_above_it() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let prizes = soroban_sdk::vec![&env, 10000u32];

    let mut config = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "max prize amount"),
        5,
        MIN_TICKET_PRICE,
        MAX_PRIZE_AMOUNT,
        prizes.clone(),
    );
    // The 1e21 bound is only reachable for non-internal randomness; internal
    // draws are capped by MAX_INTERNAL_RANDOMNESS_PRIZE_AMOUNT.
    config.randomness_source = RandomnessSource::External;
    config.oracle_address = Some(Address::generate(&env));
    client.init(&factory, &admin, &creator, &config);

    let invalid = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "max prize amount"),
        5,
        MIN_TICKET_PRICE,
        MAX_PRIZE_AMOUNT + 1,
        prizes,
    );
    assert_eq!(
        client.try_init(&factory, &admin, &creator, &invalid),
        Err(Ok(Error::InvalidParameters))
    );
}

#[test]
fn init_accepts_max_description_length_and_rejects_above_it() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let prizes = soroban_sdk::vec![&env, 10000u32];
    let inside_description = String::from_str(&env, &"a".repeat(MAX_DESCRIPTION_LENGTH as usize));
    let outside_description =
        String::from_str(&env, &"a".repeat(MAX_DESCRIPTION_LENGTH as usize + 1));

    let config = init_bounds_config(
        &env,
        &payment_token,
        inside_description,
        5,
        MIN_TICKET_PRICE,
        MIN_TICKET_PRICE * 5,
        prizes.clone(),
    );
    client.init(&factory, &admin, &creator, &config);

    let invalid = init_bounds_config(
        &env,
        &payment_token,
        outside_description,
        5,
        MIN_TICKET_PRICE,
        MIN_TICKET_PRICE * 5,
        prizes,
    );
    assert_eq!(
        client.try_init(&factory, &admin, &creator, &invalid),
        Err(Ok(Error::InvalidParameters))
    );
}

#[test]
fn init_accepts_max_prizes_and_rejects_above_it() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let mut inside_prizes = soroban_sdk::Vec::new(&env);
    for _ in 0..MAX_PRIZES {
        inside_prizes.push_back(100u32);
    }
    let mut outside_prizes = soroban_sdk::Vec::new(&env);
    for _ in 0..(MAX_PRIZES + 1) {
        outside_prizes.push_back(100u32);
    }

    let config = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "max prizes"),
        MAX_PRIZES,
        MIN_TICKET_PRICE,
        MIN_TICKET_PRICE * 5,
        inside_prizes,
    );
    client.init(&factory, &admin, &creator, &config);

    let invalid = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "max prizes"),
        MAX_PRIZES,
        MIN_TICKET_PRICE,
        MIN_TICKET_PRICE * 5,
        outside_prizes,
    );
    assert_eq!(
        client.try_init(&factory, &admin, &creator, &invalid),
        Err(Ok(Error::TooManyPrizes))
    );
}

#[test]
fn init_accepts_max_tickets_limit_and_rejects_above_it() {
    let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
    let client = ContractClient::new(&env, &contract_id);
    let prizes = soroban_sdk::vec![&env, 10000u32];

    let config = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "max tickets"),
        MAX_TICKETS_LIMIT,
        MIN_TICKET_PRICE,
        MIN_TICKET_PRICE * 5,
        prizes.clone(),
    );
    client.init(&factory, &admin, &creator, &config);

    let invalid = init_bounds_config(
        &env,
        &payment_token,
        String::from_str(&env, "max tickets"),
        MAX_TICKETS_LIMIT + 1,
        MIN_TICKET_PRICE,
        MIN_TICKET_PRICE * 5,
        prizes,
    );
    assert_eq!(
        client.try_init(&factory, &admin, &creator, &invalid),
        Err(Ok(Error::InvalidParameters))
    );
}

// ===========================================================================
// buy_tickets budget benchmark near maximum ticket count (#449)
//
// Confirms a full `max_tickets_per_tx` batch of purchases near the 100_000
// ticket ceiling stays within Soroban's per-invocation CPU/memory limits and
// still triggers the transition into `Drawing` on the final batch.
//
// NOTE: The companion `get_tickets_page_is_efficient_for_large_raffles` test
// from #449 is intentionally omitted — the raffle instance does not currently
// expose a `get_tickets_page` view, so there is no function to benchmark. It
// should be added alongside a paginated ticket-read view in a follow-up.
// ===========================================================================

#[test]
fn update_metadata_hash_before_deposit_only() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let factory = Address::generate(&env);
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = sac.address();

    let mut config = base_config(&env, &token_addr);
    config.description = String::from_str(&env, "metadata");
    config.max_tickets = 5;
    config.max_tickets_per_tx = 5;
    config.prize_amount = 50_000;
    config.metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

    client.init(&factory, &admin, &creator, &config);
    let new_hash = BytesN::from_array(&env, &[2u8; 32]);
    client.update_metadata_hash(&new_hash);
    assert_eq!(client.get_raffle().metadata_hash, new_hash);

    StellarAssetClient::new(&env, &token_addr).mint(&creator, &1_000_000);
    client.deposit_prize();
    assert_eq!(
        client.try_update_metadata_hash(&BytesN::from_array(&env, &[3u8; 32])),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn test_explicit_zero_lockup_is_honored() {
    let env = Env::default();
    env.mock_all_auths();
    let factory = Address::generate(&env);
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let payment_token = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let mut config = base_config(&env, &payment_token);
    config.description = String::from_str(&env, "Zero Lockup");
    config.max_tickets = 1;
    config.max_tickets_per_tx = 1;
    config.claim_lockup_seconds = Some(0);
    config.swap_deadline_seconds = Some(0);
    config.metadata_hash = BytesN::from_array(&env, &[99; 32]);

    client.init(&factory, &admin, &creator, &config);
    let raffle = client.get_raffle();
    
    // Explicit 0 should be stored as 0, not DEFAULT_CLAIM_LOCKUP_SECONDS
    assert_eq!(raffle.claim_lockup_seconds, 0);
    assert_eq!(raffle.swap_deadline_seconds, 0);
}

#[test]
fn test_unset_lockup_gets_default() {
    let env = Env::default();
    env.mock_all_auths();
    let factory = Address::generate(&env);
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let payment_token = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let mut config = base_config(&env, &payment_token);
    config.description = String::from_str(&env, "Default Lockup");
    config.max_tickets = 1;
    config.max_tickets_per_tx = 1;
    config.metadata_hash = BytesN::from_array(&env, &[100; 32]);

    client.init(&factory, &admin, &creator, &config);
    let raffle = client.get_raffle();
    
    // None should be resolved to the defaults
    assert_eq!(raffle.claim_lockup_seconds, DEFAULT_CLAIM_LOCKUP_SECONDS);
    assert_eq!(raffle.swap_deadline_seconds, DEFAULT_SWAP_DEADLINE_SECONDS);
}

#[test]
fn init_rejects_quorum_k_zero() {
    let (env, _, factory, admin, creator, payment_token) = init_bounds_env();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let oracle = Address::generate(&env);
    let mut oracles = Vec::new(&env);
    oracles.push_back(oracle);

    let mut config = base_config(&env, &payment_token);
    config.randomness_source = RandomnessSource::Quorum(QuorumConfig { k: 0, oracles });

    assert_eq!(
        client.try_init(&factory, &admin, &creator, &config),
        Err(Ok(Error::InvalidParameters))
    );
}

#[test]
fn init_rejects_quorum_k_greater_than_n() {
    let (env, _, factory, admin, creator, payment_token) = init_bounds_env();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let oracle = Address::generate(&env);
    let mut oracles = Vec::new(&env);
    oracles.push_back(oracle);

    let mut config = base_config(&env, &payment_token);
    config.randomness_source = RandomnessSource::Quorum(QuorumConfig { k: 2, oracles });

    assert_eq!(
        client.try_init(&factory, &admin, &creator, &config),
        Err(Ok(Error::InvalidParameters))
    );
}

#[test]
fn init_rejects_quorum_more_than_ten_oracles() {
    let (env, _, factory, admin, creator, payment_token) = init_bounds_env();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let mut oracles = Vec::new(&env);
    for _ in 0..11 {
        oracles.push_back(Address::generate(&env));
    }

    let mut config = base_config(&env, &payment_token);
    config.randomness_source = RandomnessSource::Quorum(QuorumConfig { k: 2, oracles });

    assert_eq!(
        client.try_init(&factory, &admin, &creator, &config),
        Err(Ok(Error::InvalidParameters))
    );
}

#[test]
fn init_rejects_quorum_oracle_equal_to_contract() {
    let (env, _, factory, admin, creator, payment_token) = init_bounds_env();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let mut oracles = Vec::new(&env);
    oracles.push_back(contract_id.clone());

    let mut config = base_config(&env, &payment_token);
    config.randomness_source = RandomnessSource::Quorum(QuorumConfig { k: 1, oracles });

    assert_eq!(
        client.try_init(&factory, &admin, &creator, &config),
        Err(Ok(Error::InvalidParameters))
    );
}


// ── #1228 parity: instance init and factory builder agree ───────────────────

/// Table-driven parity test: for each config mutation the instance `init` and
/// the `RaffleConfigBuilder.build()` (which calls `validate_config`) should
/// either both succeed or both fail with an equivalent error. This pins the
/// one-validator guarantee introduced in #1228.
#[test]
fn init_and_builder_accept_reject_identically() {
    use raffle_shared::config_builder::RaffleConfigBuilder;

    struct Case {
        label: &'static str,
        /// Apply a mutation to the base config before testing both paths.
        mutate: fn(&mut RaffleConfig, &Env),
        /// `true` = both should accept; `false` = both should reject.
        should_succeed: bool,
    }

    let cases: &[Case] = &[
        Case {
            label: "base config is valid",
            mutate: |_c, _e| {},
            should_succeed: true,
        },
        Case {
            label: "prize bp sum != 10000",
            mutate: |c, e| {
                c.prizes = soroban_sdk::vec![e, 5_000u32]; // only 5000 bp
            },
            should_succeed: false,
        },
        Case {
            label: "prize bp sum == 10000 with two tiers",
            mutate: |c, e| {
                c.prizes = soroban_sdk::vec![e, 6_000u32, 4_000u32];
            },
            should_succeed: true,
        },
        Case {
            label: "metadata_hash all zeros",
            mutate: |c, e| {
                c.metadata_hash = soroban_sdk::BytesN::from_array(e, &[0u8; 32]);
            },
            should_succeed: false,
        },
        Case {
            label: "claim_expiry <= claim_lockup",
            mutate: |c, _e| {
                c.claim_lockup_seconds = Some(1_000);
                c.claim_expiry_seconds = Some(1_000);
            },
            should_succeed: false,
        },
        Case {
            label: "max_tickets_per_address > max_tickets",
            mutate: |c, _e| {
                c.max_tickets_per_address = c.max_tickets + 1;
            },
            should_succeed: false,
        },
        Case {
            label: "min_tickets > max_tickets",
            mutate: |c, _e| {
                c.min_tickets = c.max_tickets + 1;
            },
            should_succeed: false,
        },
        Case {
            label: "protocol_fee_bp > 0 without treasury",
            mutate: |c, _e| {
                c.protocol_fee_bp = 100;
                c.treasury_address = None;
            },
            should_succeed: false,
        },
    ];

    for case in cases {
        let (env, contract_id, factory, admin, creator, payment_token) = init_bounds_env();
        let client = ContractClient::new(&env, &contract_id);
        let mut config = base_config(&env, &payment_token);
        (case.mutate)(&mut config, &env);

        // -- Instance path --
        let instance_result = client.try_init(&factory, &admin, &creator, &config);
        let instance_ok = instance_result.is_ok();

        // -- Builder path (validate_config) --
        let builder_result =
            RaffleConfigBuilder::from_config(&env, config.clone()).build();
        let builder_ok = builder_result.is_ok();

        assert_eq!(
            instance_ok, case.should_succeed,
            "instance init mismatch for '{}': expected {}, got {:?}",
            case.label, case.should_succeed, instance_result
        );
        assert_eq!(
            builder_ok, case.should_succeed,
            "builder mismatch for '{}': expected {}, got {:?}",
            case.label, case.should_succeed, builder_result
        );
        assert_eq!(
            instance_ok, builder_ok,
            "parity violation for '{}': instance={}, builder={}",
            case.label, instance_ok, builder_ok
        );
    }
}
