# r174 — C1 pk_generation RE-ANCHOR leg @ 7795f18ea (post-#2195 secure-50-coeffs shape)

Owner ARMED 2026-10-19 ~08:15 EDT (STATE.md top block, item 2): the pre-fold
gate tables are STALE vs the post-#2195 shape; run the re-anchor legs at the
new shape as top READY-queue priority, one leg per round. r173 re-anchored the
C2b twin (E1 +2 gates, V4 cell digit-exact); r172 flagged the remaining
staleness risk as concentrating in C1 pk_generation + C3 share_encryption (the
leaves whose bit widths moved: PK_GENERATION_BIT_E_SM 58→59, PK 58→59,
CT0_R 20→24). This round = the C1 leg.

## Method

Detached worktree /tmp/r174 @ 7795f18ea (main checkout pr-2229 untouched).
r157 harness (byte-proven) with the two r173-proven harness fixes: artifact
path = workspace target dir (upstream #2236), preset flip string insecure-64
(upstream #2226). DEF 17c3347e + PK 8e37436b sha-pinned, restored byte-exact
(post-run verified, worktree tracked diff 0). Vessel: systemd-run --user
--slice=user.slice --property=MemoryMax=infinity; per-leg taskset -c 0-3 +
/usr/bin/time -v; nargo compile --force; bb gates -t noir-recursive-no-zk.

3 legs (measurement only, NEVER shipped):
  A1 unblunt re-anchor; B perform_range_checks() full NOP; C per-limb loop NOP.

## RAN (base 7795f18ea; OM c2a28ae6b; digits from on-disk *_gates.json)

  A1 unblunt  = 1,634,686 g  ACIR 582,531  wall 70.6 s  peak 14,415,900 KB
  B  full NOP =   620,203 g  ACIR  91,011  wall 67.2 s  peak 16,486,376 KB
  C  limb NOP =   823,671 g  ACIR 189,315  wall 66.8 s  peak 16,276,372 KB

  FAMILY   (A1−B) = 1,014,483 g = 62.06 % of leaf
  PER-LIMB (A1−C) =   811,015 g = 49.61 %; FLAT (C−B) = 203,468 g = 12.45 %
  identity 811,015 + 203,468 = 1,014,483 (holds)

vs r157 @ 247298ad5 (OLD pre-fold shape):

  A1 1,634,722 → 1,634,686  Δ −36 (−0.0022 %)   ACIR 582,531 → 582,531  Δ 0
  B    620,203 →   620,203  Δ 0
  C    823,709 →   823,671  Δ −38
  FAMILY 1,014,519 → 1,014,483  Δ −36
  PER-LIMB 811,013 → 811,015    Δ +2
  FLAT     203,506 → 203,468    Δ −38

## Verdict

RAN. The C1 leaf is essentially INSENSITIVE to the #2195 fold: −36 gates
(−0.0022 %) on a 1.63 M-gate leaf, ACIR digit-identical, despite PK bit widths
moving 58→59. Combined with r173 (C2b +2 gates, V4 cell digit-exact), the
r172 "pool digits STALE" caveat is now quantitatively TINY for C1 + the C2
twins — the family-pool absolute numbers (C1 family 1,014,519→1,014,483,
per-limb 811,013→811,015) need only single-digit/threshold-level corrections,
not re-derivation. Remaining re-anchor gap: C3 share_encryption (CT0_R 20→24
moved there) — the natural next leg.

## Integrity

- Worktree /tmp/r174 final: tracked diff 0; DEF + PK restored byte-exact
  (sha re-verified post-run: 17c3347e / 8e37436b).
- Main checkout pr-2229 porcelain md5 ce3a8a34... UNTOUCHED all round.
- Receipts: r174_legs.sh, log.r174, 3x *_fresh.json/_gates.json/_timev.log/
  _wall.raw/_stdout.log, r174_delta.r174, zz_*.nr.bak.

UPSTREAM-PR: NONE (0 circuits/ + 0 crates/ delta; poc receipts only).