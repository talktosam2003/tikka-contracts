//! Raffle-instance initialisation and prize-deposit logic.
//!
//! This module contains the two functions that move a raffle from *nothing*
//! into a state where ticket sales can begin:
//!
//! 1. [`init`] — called once by the factory immediately after deployment.
//!    Validates every field of [`RaffleConfig`], writes the [`Raffle`] to
//!    instance storage, and emits [`events::RaffleCreated`].
//!
//! 2. [`deposit_prize`] — called by the creator after `init`.  Transfers the
//!    prize amount from the creator's wallet into the contract and transitions
//!    the raffle from [`RaffleStatus::PendingPrize`] to
//!    [`RaffleStatus::Active`], opening ticket sales.
//!
//! ## Raffle lifecycle
//!
//! ```text
//! [deploy] ──init()──► PendingPrize ──deposit_prize()──► Active
//!                                                           │
//!                                                  ticket sales open
//! ```
//!
//! See [`docs/RANDOMNESS.md`](../../../docs/RANDOMNESS.md) for a full
//! explanation of the three randomness modes that can be configured here, and
//! [`docs/EVENTS.md`](../../../docs/EVENTS.md) for the events emitted by
//! these functions.

use soroban_sdk::{token, Env};

use crate::events::PrizeDeposited;
use crate::{
    helpers::{read_raffle, require_not_paused, apply_status_transition},
    write_raffle, Error, RaffleStatus,
};

/// Transfer the prize amount from the creator into the contract and open ticket
/// sales.
///
/// This is the second mandatory setup step after [`init`].  Until this
/// function succeeds, [`buy_tickets`] will return
/// [`Error::InvalidStateTransition`] because `prize_deposited` is `false`.
///
/// ## What this function does
///
/// 1. Checks the contract is not paused.
/// 2. Requires authorization from `raffle.creator`.
/// 3. Guards against a second deposit (`prize_deposited == true`).
/// 4. Calls `try_transfer` on the prize token to pull `prize_amount` from
///    the creator into this contract address.
/// 5. Sets `prize_deposited = true` and transitions status from
///    [`RaffleStatus::PendingPrize`] → [`RaffleStatus::Active`].
///
/// After this call succeeds, `buy_tickets` accepts purchases.
///
/// # Auth
///
/// Requires authorization from `raffle.creator`.
///
/// # Errors
///
/// - [`Error::ContractPaused`] — contract is paused.
/// - [`Error::NotInitialized`] — `init` has not been called yet.
/// - [`Error::PrizeAlreadyDeposited`] — prize was already deposited; calling
///   again is a no-op error to prevent double-funding.
/// - [`Error::TokenTransferFailed`] — the token `try_transfer` failed (e.g.
///   insufficient creator balance or missing allowance).
///
/// # Events
///
/// - [`events::PrizeDeposited`] — confirms the amount and token.
/// - [`events::RaffleStatusChanged`] — records the `PendingPrize → Active`
///   transition.
///
/// See also: [`docs/EVENTS.md`](../../../docs/EVENTS.md) — `PrizeDeposited`,
/// `RaffleStatusChanged`.
pub(crate) fn deposit_prize(env: Env) -> Result<(), Error> {
    require_not_paused(&env)?;
    let mut raffle = read_raffle(&env)?;
    raffle.creator.require_auth();

    if raffle.prize_deposited {
        return Err(Error::PrizeAlreadyDeposited);
    }

    let token_client = token::Client::new(&env, &raffle.prize_token);
    let _ = token_client
        .try_transfer(&raffle.creator, env.current_contract_address(), &raffle.prize_amount)
        .map_err(|_| Error::TokenTransferFailed)?;

    raffle.prize_deposited = true;
    let ts = env.ledger().timestamp();
    apply_status_transition(&env, &mut raffle, RaffleStatus::Active, ts)?;
    write_raffle(&env, &raffle);

    PrizeDeposited { creator: raffle.creator.clone(), amount: raffle.prize_amount, token: raffle.prize_token.clone(), timestamp: ts }.publish(&env);

    Ok(())
}
