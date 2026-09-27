//! Optional hash-keyed snapshot cache helpers (Issue #660).
//!
//! A caller must invalidate on every configuration write before using this
//! cache. Existing contract entrypoints do not use it; compiling this module
//! does not add writes to read-only contract methods.

use soroban_sdk::{contracttype, symbol_short, Env, Symbol};

use crate::SLAConfigSnapshot;

/// On-chain storage key for the cached `(hash, snapshot)` pair.
pub(crate) const SNAPSHOT_CACHE_KEY: Symbol = symbol_short!("SNPCCH");

/// Cached snapshot entry — stores the hash alongside the snapshot so a
/// single storage read determines whether the cache is still valid.
#[contracttype]
#[derive(Clone)]
pub struct SnapshotCacheEntry {
    pub config_version_hash: u64,
    pub snapshot: SLAConfigSnapshot,
}

/// Return the cached snapshot if the config hash is unchanged,
/// otherwise `None` (caller must recompute and call `store_snapshot_cache`).
pub fn read_snapshot_cache(env: &Env, current_hash: u64) -> Option<SLAConfigSnapshot> {
    let entry: SnapshotCacheEntry = env.storage().instance().get(&SNAPSHOT_CACHE_KEY)?;
    if entry.config_version_hash == current_hash {
        Some(entry.snapshot)
    } else {
        None
    }
}

/// Persist a freshly built snapshot alongside its hash.
/// Called after every recompute so subsequent reads are cache-hits.
pub fn store_snapshot_cache(env: &Env, hash: u64, snapshot: SLAConfigSnapshot) {
    env.storage().instance().set(
        &SNAPSHOT_CACHE_KEY,
        &SnapshotCacheEntry {
            config_version_hash: hash,
            snapshot,
        },
    );
}

/// Invalidate the snapshot cache.
/// Must be called by every config-write path so the next read rebuilds.
pub fn invalidate_snapshot_cache(env: &Env) {
    env.storage().instance().remove(&SNAPSHOT_CACHE_KEY);
}
