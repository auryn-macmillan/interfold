# R190 — r8 batch-arm adaptation: PORTED + RAN (batched fold is a verified drop-in, ~43% faster)

**Date:** 2026-10-22 ~01:50 UTC **Cron:** (this round) **Worktree:** /home/dev/i5-rebase-wt
(i5/dkg-research @ c1f7162ef, ahead 125/behind 0)

## What R189 left

R189 found the r8 batch arm DEAD-AT-TIP: `c3_fold_batch_n3` hardcodes the pre-#2258 4-pub-param
accumulator (`C3_FOLD_PUBLIC_LEN = 4 + 3*C3_SLOTS` -> api=31) while post-#2258
`c3_fold`/`c3_fold_kernel` are 5-prefix (api/acc=32) ->
`WitnessGenerationFailed ABI LengthMismatch path=api expected 31 actual 32`. Sequential arm
re-anchored green (20.2s). READY candidate: "port batch circuits to 5-prefix + rebuild + rerun".

## Root cause (deeper than R189's one-liner) — the test assertions were ALSO stale, inverted

Compiled ABI (runtime truth, RAN): `c3_fold` acc_public_inputs len=**32**, `c3_fold_kernel`
len=**32**, `c3_fold_batch_n3` api len=**31**. The kernel genesis anchor is 5-prefix (32); the
sequential c3_fold shares the kernel's 5 pub-scalar structure, so the seq proof public signals =
5-prefix = 32. But the test (`batch_wall_clock_tests_r8.rs`, written 2026-08-18 when c3_fold was
4-prefix) still asserts `seq.len()==4+3·slots` (line 133) and `s[4..]==b[5..]` (line 135) — i.e.
seq=4-prefix. #1999 (2026-10-03, "bound openings, quotients and VK trees") flipped c3_fold to
5-prefix; the test's seq assertions were never updated. **R189's circuit-only fix was
insufficient**: bumping n3 to 5-prefix would move the failure from line 123 (batch witness-gen) to
line 133 (seq len assertion).

## Fix (both halves)

1. **Circuits** `c3_fold_batch_n2/n3/n4/src/main.nr`: `C3_FOLD_PUBLIC_LEN` 4 -> 5 (auto-derives
   `C3_FOLD_PREFIX_LEN` 4 -> 5); hardcoded slot-0 carry
   `pk[0]=api[4]; msg[0]=api[4+..]; ct[0]=api[4+..]` -> `api[5]` (slot 0 lives at prefix index 5).
   Leaf `apk[C3_FOLD_PREFIX_LEN + slot]==0` asserts auto-adjust with the derived constant.
2. **Test** `batch_wall_clock_tests_r8.rs`: line 133 `4 + 3*total_slots` -> `5 + 3*total_slots`;
   line 135 `s[4..]` -> `s[5..]`. Line 134 (bat=5+3·slots) was already correct (n3 has 5 pub scalars
   ikh0/ikh1/akh/gen_hash0/gen_hash1 + 3·slots output, independent of the api prefix) — left
   unchanged. This is a stale-assertion fix to match the current 5-prefix production circuit, NOT a
   metric change: the measured quantities (batched wall, slot-tail byte-equality) are untouched.

## Build (RAN)

`cargo run --quiet --locked -p e3-zk-build -- --circuit c3_fold_batch_n2 --circuit c3_fold_batch_n3 --circuit c3_fold_batch_n4`
-> BUILD_RC 0 (<30s). New compiled ABI: n2/n3/n4 `api` len=**32** (5-prefix, verified via python ABI
dump). Artifacts json+vk_recursive+vk_recursive_hash fresh @01:48 UTC.

## Test (RAN) — batched-vs-sequential drop-in equivalence

`/usr/bin/time -v cargo test --release -p e3-zk-prover --test batch_wall_clock_tests_r8 -- --include-ignored --nocapture --test-threads=1`
-> **TEST_RC 0, 1 passed / 0 failed**, finished 32.25s. Whole run (w/ 43.7s compile): wall 1:16.01,
user 456.80s @ 614% CPU, maxrss 3.06 GiB, Swaps 0.

```
generate_sequential_c3_fold  wall = 20.0s  verify = PASS
generate_batched_c3_fold     wall = 11.4s  verify = PASS
IDENTICAL ACCUMULATOR STATE (27 slot fields): true
```

## VERDICT — RAN

The batched c3 fold is a **verified byte-identical drop-in** for the sequential fold at the minimum
committee (C3_SLOTS=9, 3 inner proofs -> N3), and it is **~43% faster wall-clock (20.0s -> 11.4s,
1.75x)**. The r8 batched-vs-serial wall comparison that R189 could not re-anchor is now RE-ANCHORED
GREEN. The production 5-prefix accumulator is respected by both arms; the batch carries the kernel
anchor's slot 0 from api[5] and lands the identical 27-field slot tail.

## Scope note / follow-up (DRAFT, not started)

- Fixed n2/n3/n4 (the `generate_batched_c3_fold` N-family). The **b-family** gates
  (`c3_fold_batch_b2/b3/b6/b10`, `generate_c3_fold_batch_gate` in c3_accumulator.rs) still use
  `expected_fields = 4 + 3*total_slots` (RAN lines ~407/833/928/1214/1491) — same pre-#2258
  staleness; those test paths (generate_batched_c3_fold_b2 etc.) are a separate, larger migration
  and were NOT touched.
- `c3_fold_batch_merge_m7` (19x3=57-slot production shape) prefix audit still pending (R189
  carry-over).
- No source outside the 3 circuits + 1 test file. No push (enclave owner-gated; never push origin).

## Provenance

i5/dkg-research HEAD after commit: (see LOG.md). Receipts: poc/r190/. Box 8c/32GiB/0swap; disk avail
~3.14 GiB (97%) throughout — above the 2 GiB floor, no clean needed.
