//! Two-step admin and operator transfer governance.
//!
//! This module implements the two-step handoff pattern for admin and operator
//! role transfers, plus admin renounce and single-step operator assignment.
//! All functions require the appropriate role authorization and emit versioned
//! governance events for backend audit trails.

use soroban_sdk::{Address, Env, Symbol};

use crate::{
    SLAError, ADMIN_KEY, EVENT_ADMIN_ACC, EVENT_ADMIN_CAN, EVENT_ADMIN_PROP, EVENT_ADMIN_REN, EVENT_OP_ACC,
    EVENT_OP_CAN, EVENT_OP_PROP, EVENT_OP_SET, EVENT_VERSION, OPERATOR_KEY, PENDING_ADMIN_KEY,
    PENDING_ADMIN_TS_KEY, PENDING_OP_KEY, PENDING_OP_TS_KEY,
};

/// Window (in ledger seconds) after which a pending proposal expires.
const PROPOSAL_EXPIRY_WINDOW: u64 = 90 * 24 * 60 * 60;

/// Requires that the stored proposal is still within its expiry window.
fn require_proposal_valid(env: &Env, ts_key: Symbol) -> Result<(), SLAError> {
    let proposed: u64 = env
        .storage()
        .instance()
        .get(&ts_key)
        .ok_or(SLAError::NoPendingTransfer)?;
    let now = env.ledger().timestamp();
    if now.saturating_sub(proposed) > PROPOSAL_EXPIRY_WINDOW {
        return Err(SLAError::ProposalExpired);
    }
    Ok(())
}

/// Proposes a new admin. The current admin initiates; the new admin must
/// call `accept_admin` to complete the transfer.
pub fn propose_admin(env: &Env, caller: &Address, new_admin: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    crate::SLACalculatorContract::require_admin(env, caller)?;
    env.storage().instance().set(&PENDING_ADMIN_KEY, new_admin);
    env.storage().instance().set(&PENDING_ADMIN_TS_KEY, &env.ledger().timestamp());
    env.events().publish(
        (EVENT_ADMIN_PROP, EVENT_VERSION, caller.clone()),
        (new_admin.clone(),),
    );
    Ok(())
}

/// Accepts a pending admin transfer. Must be called by the proposed new admin.
pub fn accept_admin(env: &Env, caller: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    caller.require_auth();
    let pending: Address = env
        .storage()
        .instance()
        .get(&PENDING_ADMIN_KEY)
        .ok_or(SLAError::NoPendingTransfer)?;
    require_proposal_valid(env, PENDING_ADMIN_TS_KEY)?;
    if *caller != pending {
        return Err(SLAError::Unauthorized);
    }
    env.storage().instance().set(&ADMIN_KEY, caller);
    env.storage().instance().remove(&PENDING_ADMIN_KEY);
    env.storage().instance().remove(&PENDING_ADMIN_TS_KEY);
    env.events()
        .publish((EVENT_ADMIN_ACC, EVENT_VERSION, caller.clone()), ());
    Ok(())
}

/// Cancels a pending admin proposal. Only the current admin may cancel.
pub fn cancel_admin_proposal(env: &Env, caller: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    crate::SLACalculatorContract::require_admin(env, caller)?;
    if !env.storage().instance().has(&PENDING_ADMIN_KEY) {
        return Err(SLAError::NoPendingTransfer);
    }
    env.storage().instance().remove(&PENDING_ADMIN_KEY);
    env.storage().instance().remove(&PENDING_ADMIN_TS_KEY);
    env.events()
        .publish((EVENT_ADMIN_CAN, EVENT_VERSION, caller.clone()), ());
    Ok(())
}

/// Returns the pending admin address, if any.
pub fn get_pending_admin(env: &Env) -> Result<Option<Address>, SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    Ok(env.storage().instance().get(&PENDING_ADMIN_KEY))
}

/// Proposes a new operator. The current admin initiates; the new operator
/// must call `accept_operator` to complete the handoff.
pub fn propose_operator(env: &Env, caller: &Address, new_operator: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    crate::SLACalculatorContract::require_admin(env, caller)?;
    env.storage().instance().set(&PENDING_OP_KEY, new_operator);
    env.storage().instance().set(&PENDING_OP_TS_KEY, &env.ledger().timestamp());
    env.events().publish(
        (EVENT_OP_PROP, EVENT_VERSION, caller.clone()),
        (new_operator.clone(),),
    );
    Ok(())
}

/// Accepts a pending operator handoff. Must be called by the proposed new operator.
pub fn accept_operator(env: &Env, caller: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    caller.require_auth();
    let pending: Address = env
        .storage()
        .instance()
        .get(&PENDING_OP_KEY)
        .ok_or(SLAError::NoPendingTransfer)?;
    require_proposal_valid(env, PENDING_OP_TS_KEY)?;
    if *caller != pending {
        return Err(SLAError::Unauthorized);
    }
    env.storage().instance().set(&OPERATOR_KEY, caller);
    env.storage().instance().remove(&PENDING_OP_KEY);
    env.storage().instance().remove(&PENDING_OP_TS_KEY);
    env.events()
        .publish((EVENT_OP_ACC, EVENT_VERSION, caller.clone()), ());
    Ok(())
}

/// Cancels a pending operator proposal. Only the current admin may cancel.
pub fn cancel_operator_proposal(env: &Env, caller: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    crate::SLACalculatorContract::require_admin(env, caller)?;
    if !env.storage().instance().has(&PENDING_OP_KEY) {
        return Err(SLAError::NoPendingTransfer);
    }
    env.storage().instance().remove(&PENDING_OP_KEY);
    env.storage().instance().remove(&PENDING_OP_TS_KEY);
    env.events()
        .publish((EVENT_OP_CAN, EVENT_VERSION, caller.clone()), ());
    Ok(())
}

/// Returns the pending operator address, if any.
pub fn get_pending_operator(env: &Env) -> Result<Option<Address>, SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    Ok(env.storage().instance().get(&PENDING_OP_KEY))
}

/// Permanently renounces admin authority. Irreversible.
pub fn renounce_admin(env: &Env, caller: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    crate::SLACalculatorContract::require_admin(env, caller)?;
    env.storage().instance().remove(&ADMIN_KEY);
    env.storage().instance().remove(&PENDING_ADMIN_KEY);
    env.storage().instance().remove(&PENDING_ADMIN_TS_KEY);
    env.events()
        .publish((EVENT_ADMIN_REN, EVENT_VERSION, caller.clone()), ());
    Ok(())
}

/// Replaces the operator address directly (single-step, admin only).
pub fn set_operator(env: &Env, caller: &Address, new_operator: &Address) -> Result<(), SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    crate::config_freeze::require_not_frozen(env)?;
    crate::SLACalculatorContract::require_admin(env, caller)?;
    env.storage().instance().set(&OPERATOR_KEY, new_operator);
    env.events().publish(
        (EVENT_OP_SET, EVENT_VERSION, caller.clone()),
        (new_operator.clone(),),
    );
    Ok(())
}
