// SPDX-License-Identifier: LGPL-3.0-only
//
//! r130 regression: fixture-bootstrap nested-cargo deadlock (r127 n2, composite fix).
//!
//! Deadlock mechanism (r127 RAN diagnosis): `crates/zk/prover/build.rs` runs
//! `scripts/build_fixtures.sh`, which — when the 3 recursive_aggregation fold
//! artifacts (c3_fold / c6_fold / c6_fold_kernel json) are missing from a lean
//! checkout — invokes `pnpm build:circuits`. Pre-fix, that builder ran a NESTED
//! `cargo run --release --bin generate_parity_matrices` in the SAME target dir
//! while the OUTER `cargo test`/`cargo build` process still held
//! `target/release/.cargo-lock`. Outer waits on build.rs; build.rs waits on
//! pnpm; pnpm waits on the nested cargo; the nested cargo waits on the outer
//! lock. Deadlock: the first `cargo test` (or `cargo build`) of the zk-prover
//! crate on a lean checkout could NEVER complete (r127 observed this via a
//! second outer job hang).
//!
//! R178 FOLD-FORWARD UPDATE: upstream #2260 ("move circuit build ownership
//! into Rust") deleted `scripts/build-circuits.ts`; `pnpm build:circuits` is
//! now `cargo run --quiet --locked -p e3-zk-build --`. The TS-side guards of
//! the r130 composite fix (FIX1 `--skip-regen-parity`, FIX3 `isLibOnly`)
//! died with the file and are obsolete: the Rust builder regenerates parity
//! matrices in-process (no nested `cargo run` call site exists anywhere in
//! the chain any more — see the negative-core assertion below). The
//! deadlock-class defense that REMAINS load-bearing is FIX2: the
//! CARGO_TARGET_DIR redirect in build_fixtures.sh, now carried by upstream
//! itself. This test is retained (r130 evidence) re-anchored to the
//! post-#2260 shape: FIX2 redirect present + no nested-cargo call site in
//! the bootstrap chain.
//!
//! RAN proof of the original fix (see poc/r130/RESULT.txt + v5_run.log):
//!   (a) the EXACT deadlocking call pattern still deadlocked at r130 with a
//!       pre-fix-shaped invocation (nested `cargo run` under an outer cargo in
//!       the same target dir), and
//!   (b) the FIXED bootstrap drove a lean stage tree missing all required
//!       fold artifacts through 36/36 circuit compiles with ZERO nested
//!       `cargo` processes in the dead context. No hang.

#[test]
fn r130_bootstrap_redirects_nested_cargo_off_outer_target_dir() {
    // The test binary runs with cwd = crate root, but the repo paths the
    // preload fix touches are workspace-root-relative — resolve from the
    // manifest dir (the same way build.rs reaches the same script).
    let root = std::path::Path::new(std::env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let sh = std::fs::read_to_string(root.join("crates/zk/prover/scripts/build_fixtures.sh"))
        .unwrap_or_else(|e| panic!("crates/zk/prover/scripts/build_fixtures.sh unreadable: {e}"));

    // FIX2 (load-bearing, now upstream-carried): the in-cargo bootstrap must
    // redirect CARGO_TARGET_DIR off the outer cargo's target directory
    // (post-2026-09-24-rebase shape; matches the upstream independent fix of
    // the same deadlock, #1984). A nested cargo with the SAME target dir
    // parks on the outer target's .cargo-lock forever (r127 n2 / r130) — any
    // in-cargo invocation of build:circuits without the redirect reopens the
    // class.
    assert!(
        sh.contains("CARGO_TARGET_DIR=\"${CARGO_TARGET_DIR:-$PWD/target}/e3-zk-prover-fixtures\" pnpm build:circuits"),
        "build_fixtures.sh lost the CARGO_TARGET_DIR redirect: the in-cargo \
         fixture bootstrap would deadlock on target/.cargo-lock again (r127 n2 / r130)"
    );

    // Negative core (post-#2260 re-anchor): the deadlock's original call site
    // was a nested `cargo run` inside the TS builder. The TS builder is gone;
    // the chain must NOT reintroduce any nested `cargo run`/`cargo build`
    // call in build_fixtures.sh itself (pnpm build:circuits IS a cargo run —
    // that is the one cargo the redirect protects; nothing else may spawn
    // cargo from inside this script).
    let cargo_calls: Vec<&str> = sh
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter(|l| l.contains("cargo run") || l.contains("cargo build"))
        .collect();
    assert!(
        cargo_calls.is_empty(),
        "build_fixtures.sh reintroduces a direct nested cargo call outside the \
         redirected pnpm build:circuits path (r130 deadlock class): {cargo_calls:?}"
    );

    println!("PASS r130: bootstrap chain keeps the CARGO_TARGET_DIR redirect, no direct nested cargo (post-#2260 shape)");
}