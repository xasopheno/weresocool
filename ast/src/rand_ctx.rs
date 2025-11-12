//! Tiny, splittable randomness for arrays & their children (single file)
//! -----------------------------------------------------------------------------
//! Goal: **Given an array**, derive a fresh, deterministic randomness context
//! for each item — and all of that item's *children* automatically inherit it.
//! No global RNG, no IDs required, pure & lazy-friendly.
//!
//! Core API you'll use:
//! - `RandCtx::from_u128(seed)` — make a root context once
//! - `let items = root.split("items");` — start a scope for the array
//! - `let ctx_i = items.item(i);` — per-item context (unique & deterministic)
//! - Inside the item subtree, just pass `ctx_i` down; children call
//!   `child_ord(k)` (or more splits) as needed.
//!
//! That's it. Everything below is the tiny implementation.

#![allow(dead_code)]

// =============================================================================
// Core context (tiny, Copy-like footprint)
// =============================================================================
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RandCtx {
    seed:  u128, // global run seed
    key:   u128, // path/address key (derived as you traverse)
    index: u64,  // which item/reuse/call/frame
    epoch: u32,  // resample knob for a scope
}

impl RandCtx {
    /// Root context from a 128-bit seed (e.g., hash of project/run id).
    pub const fn from_u128(seed: u128) -> Self {
        Self { seed, key: 0, index: 0, epoch: 0 }
    }

    /// Per-item/call/frame index (cheap copy).
    #[inline] pub const fn with_index(self, index: u64) -> Self { Self { index, ..self } }
    /// Resample everything under this context (wrapping bump).
    #[inline] pub const fn bump_epoch(self) -> Self { Self { epoch: self.epoch.wrapping_add(1), ..self } }

    /// Derive a new path key by mixing any tag (opcode name, site label,…).
    #[inline] pub fn with_key<T: core::hash::Hash>(self, tag: T) -> Self { Self { key: mix128(self.key, tag), ..self } }
    /// Structure-only split (no hashing of strings), by ordinal.
    #[inline] pub fn child_ord(self, k: u64) -> Self { self.with_key(k) }

    // ---------------- Random draws (pure PRF over {seed,key,index,epoch,salt})

    /// 64 random bits deterministic on (seed,key,index,epoch,salt).
    #[inline] pub fn u64(self, salt: u64) -> u64 {
        let (s_hi, s_lo) = split_u128(self.seed);
        let (k_hi, k_lo) = split_u128(self.key);
        let mut x = 0u64;
        x = mix64(x ^ s_hi);
        x = mix64(x ^ s_lo);
        x = mix64(x ^ k_hi);
        x = mix64(x ^ k_lo);
        x = mix64(x ^ self.index);
        x = mix64(x ^ (self.epoch as u64));
        x = mix64(x ^ salt);
        x
    }

    /// Uniform double in [0,1).
    #[inline] pub fn f64_01(self, salt: u64) -> f64 {
        let x = self.u64(salt) >> 11; // top 53 bits
        (x as f64) * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in [lo, hi).
    #[inline] pub fn uniform_f64(self, lo: f64, hi: f64, salt: u64) -> f64 {
        debug_assert!(hi >= lo, "uniform_f64: hi < lo");
        lo + (hi - lo) * self.f64_01(salt)
    }

    /// Integer in [0, n) without modulo bias (rejection sampling).
    #[inline]
    pub fn index(self, n: core::num::NonZeroUsize, salt: u64) -> usize {
        let n64 = n.get() as u64;
        let lim = u64::MAX - u64::MAX % n64;
        let mut r = self.u64(salt);
        let mut i = 0u64;
        while r >= lim { i = i.wrapping_add(0x9E37_79B9_7F4A_7C15); r = mix64(r ^ i ^ salt); }
        (r % n64) as usize
    }

    // ---------------- Ergonomic array-scoping -------------------------------

    /// Start a scope for an array/list of items (e.g., notes, shapes, clips).
    /// Children will inherit this key; each item gets a unique index below.
    #[inline] pub fn split<T: core::hash::Hash>(self, tag: T) -> RandArray { RandArray { base: self.with_key(tag) } }
}

/// A tiny wrapper for "randomness per array item".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RandArray { base: RandCtx }
impl RandArray {
    /// Context for item `i` (unique and deterministic).
    #[inline] pub fn item(self, i: u64) -> RandCtx { self.base.with_index(i) }
    /// Optional: resample the whole array scope (fresh draws under it).
    #[inline] pub fn resampled(self) -> RandArray { RandArray { base: self.base.bump_epoch() } }
    /// Split again structurally for sub-arrays/children of each item.
    #[inline] pub fn split_child_ord(self, k: u64) -> RandArray { RandArray { base: self.base.child_ord(k) } }
    /// Or split with a tag if you have one.
    #[inline] pub fn split_child<T: core::hash::Hash>(self, tag: T) -> RandArray { RandArray { base: self.base.with_key(tag) } }
}

// =============================================================================
// Lightweight mixers (no std deps)
// =============================================================================
#[inline] fn split_u128(x: u128) -> (u64, u64) { ((x >> 64) as u64, x as u64) }
#[inline] fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30; x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27; x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}
fn mix128<T: core::hash::Hash>(parent: u128, label: T) -> u128 {
    use core::hash::{Hash, Hasher};
    struct H(u64);
    impl Hasher for H {
        fn write(&mut self, bytes: &[u8]) {
            let mut x = self.0;
            for chunk in bytes.chunks(8) {
                let mut buf = [0u8; 8];
                for (i, b) in chunk.iter().enumerate() { buf[i] = *b; }
                x = mix64(x ^ u64::from_le_bytes(buf));
            }
            self.0 = x;
        }
        fn write_u64(&mut self, i: u64) { self.0 = mix64(self.0 ^ i); }
        fn finish(&self) -> u64 { self.0 }
    }
    let (p_hi, p_lo) = split_u128(parent);
    let mut h1 = H(0x243F_6A88_85A3_08D3); p_hi.hash(&mut h1); p_lo.hash(&mut h1); label.hash(&mut h1);
    let mut h2 = H(0x1319_E379_37F4_A7C1); label.hash(&mut h2); p_lo.hash(&mut h2); p_hi.hash(&mut h2);
    ((h1.finish() as u128) << 64) | (h2.finish() as u128)
}

// =============================================================================
 // Example usage (keep while integrating, then remove if you like)
// =============================================================================
#[cfg(test)]
mod demo {
    use super::*;

    #[test]
    fn per_item_is_unique_and_stable() {
        let root = RandCtx::from_u128(0xFEED_FACE_CAFE_BEEF);
        let items = root.split("notes");
        let a0 = items.item(0).u64(0);
        let a1 = items.item(1).u64(0);
        assert_ne!(a0, a1);                   // different items → different draws
        assert_eq!(a0, items.item(0).u64(0)); // same item again → same draw
    }

    #[test]
    fn children_inherit_item_context() {
        let root = RandCtx::from_u128(123);
        let items = root.split("shapes");
        // item 3, child 0 vs child 1 are independent but tied to item 3
        let i3 = items.item(3);
        let c0 = i3.child_ord(0).u64(7);
        let c1 = i3.child_ord(1).u64(7);
        assert_ne!(c0, c1);
        // Recomputing with same structure yields the same results
        let i3b = items.item(3);
        assert_eq!(c0, i3b.child_ord(0).u64(7));
        assert_eq!(c1, i3b.child_ord(1).u64(7));
    }
}
