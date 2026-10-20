# R189 — r8 kernel-group re-anchor, leg 1 (2026-10-20 ~23:15-23:45 UTC)

Base: i5/dkg-research 4b852d2b6 @ origin/main ab6b9c943 (ahead 125 / behind 0, worktree porcelain
0). Box: 8c / 32,095 MiB / Swap 0; nargo 1.0.0-beta.26 (git 40d6574f), bb 5.2.0.

## Setup RAN

- Fresh `dkg/share_encryption` insecure-64/minimum build at tip:
  `cargo run --release -p e3-zk-build -- build --preset insecure-64 --committee minimum --group dkg --circuit share_encryption`
  BUILD_RC 0 (~2 min). Artifacts: circuits/bin/dkg/target/share_encryption.{json (271,310 B), vk,
  vk_hash, vk_noir, vk_noir_hash, vk_recursive, vk_recursive_hash} @ 2026-10-20 23:17 UTC. Builder
  in-place configs/\*.nr patches RESTORED (porcelain 0 after).
- Fold artifacts already in-tree and fresh (c3_fold / c3_fold_kernel / c3_fold_batch_n3
  json+vk_recursive, built 2026-10-19 19:01 UTC, post-fold#4, git-ignored).

## Leg RAN

`/usr/bin/time -v cargo test --release -p e3-zk-prover --test batch_wall_clock_tests_r8 -- --nocapture --test-threads=1`
TEST_RC 101. Full log: r189_r8_test.log (copied here).

- SEQUENTIAL arm RAN GREEN: `generate_sequential_c3_fold` (genesis kernel fold + 3 step folds,
  InsecureThreshold64 minimum committee) wall = 20.2 s, verify = PASS. Whole-run maxrss 2,015,532 kB
  (1.92 GiB), Swaps 0, 2:38.72 wall incl. compile (user 1083.83 s @ 713% CPU). => NEW-BOX re-anchor
  number for the r8 serial 3-fold leg: 20.2 s @8c (r8-era 4c numbers live only in the lost
  research-repo history + poc/r8_dropin_run.log).
- BATCHED arm FAILED at tip — REAL FINDING, not harness noise:
  `Witness generation failed: ABI encode: TypeMismatch(LengthMismatch { path: "api", expected_length: 31, actual_length: 32 })`
  at batch_wall_clock_tests_r8.rs:123.

## Root cause (RAN, source-level)

- Post-#2258 `circuits/bin/recursive_aggregation/c3_fold/src/main.nr:23`:
  `C3_FOLD_PUBLIC_LEN = 5 + (3 * C3_SLOTS)` (FIVE public params; PREFIX_LEN 5).
- poc-era `c3_fold_batch_n3/src/main.nr` (round 4.4 header):
  `C3_FOLD_PUBLIC_LEN = 4 + (3 * C3_SLOTS)` (FOUR public params; slot-0 carry `pk[0] = api[4]`
  hardcodes the 4-prefix).
- At minimum committee C3_SLOTS = 9 => batch circuit ABI api = 4+27 = 31; post-#2258 witness emits
  5+27 = 32 => exact observed mismatch. Compiled c3_fold_batch_n3.json ABI confirms api length 31.
- The whole poc batch family carries the stale 4-prefix (c3_fold_batch_n2/n3/n4 same header;
  c3_fold_batch_merge_m7 cites the 19x3=57-slot production shape and needs the same audit).

## Verdict

r8 kernel-group re-anchor: SEQUENTIAL leg RE-ANCHORED (20.2 s PASS @8c, Swaps 0, 1.92 GiB peak);
BATCHED leg DEAD-AT-TIP since #2258 — receipt r8's batched-vs-serial wall comparison cannot be
re-anchored until the batch poc circuits are adapted to the 5-prefix accumulator layout.

## READY queue update

NEW scoped candidate (not owner-gated): "r8 batch-arm adaptation" — port c3_fold_batch_n3
(+n2/n4/merge_m7 audit) to the post-#2258 5-pub-param accumulator (prefix 4->5, api[4]->api[5]
carry, new commitment-convention asserts per #2258), rebuild insecure-64 artifacts, re-run
batch_wall_clock_tests_r8 for the batched wall. Est: 1 round (build ~2-20 min + test). NOT started
this round (budget spent on leg 1 + provenance incident recovery).

## Provenance incident (same round)

The research repo .git (/home/dev/interfold-dkg-research/.git) is GONE — dotfile-skipping copy into
/home/dev/interfold-research/ left .git with the deleted source. All STATE.md/LOG.md commit ids
(d1d49a43 etc.) unreachable; file content intact. Recovery: narrow git init at
/home/dev/interfold-research, baseline commit fe67f39 (.gitignore `/*`, whitelists STATE.md, LOG.md,
notes/, templates/). STATE.md md5 34d0d56e3d42245a41e59a372e5b5073 (1,100,958 B), LOG.md md5
2c121d052d6f0585c8f1b1ddf1c2313f (5,896,952 B) at baseline. PoC provenance (i5 repo:
i5/dkg-research + research/\* branches, 1698 commits, enclave remote) UNAFFECTED.

## Residual

- Disk post-leg: df avail 3,551,412 kB (~3.4 GiB, 97%); target/ grew ~1.6 GiB (release test build).
  Above 2 GiB floor, watch.
- No circuits/ or crates/ source edits this round (receipts only). Enclave push owner-gated, NO
  push; never push origin; no upstream PR.
