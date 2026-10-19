#!/usr/bin/env bash
# r180 - re-stage the secure-8192/micro C2 pair at the POST-FOLD base (fresh nargo compile
# via e3-zk-build, NOT the Sep-1 r82 pins) + fresh noir-recursive write_vk -> stage tree for
# c2_micro_prove_tests_r82. The r82 stage tree (sha 96aae9c6/588dbfa7, built 2026-09-01)
# PREDATES #1999 (checked-commitment asserts, 2026-10-03) and is INCOMPATIBLE with the
# post-#2258 witness crate: prove fails WitnessGenerationFailed("Failed assertion").
# Usage: stage_r180_micro.sh   (build ~18 min, then 2x write_vk ~40 s)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$HOME/.nargo/bin:$PATH"
WT=/home/dev/i5-rebase-wt
ART=/tmp/r180_art
OUT=/home/dev/interfold-research/poc/r180/root/secure-8192/micro/recursive/dkg
cd "$WT"
# 1) fresh compile (builder patches configs/*.nr in place - restore after)
systemd-run --user --scope --property=MemoryMax=infinity -- \
  cargo run --release -p e3-zk-build -- build --preset secure-8192 --committee micro \
  --circuit sk_share_computation --circuit e_sm_share_computation \
  -o "$ART" --skip-vk --skip-checksums
git checkout -- circuits/lib/src/configs/committee/active.nr \
  circuits/lib/src/configs/default/mod.nr \
  circuits/lib/src/configs/secure/threshold.nr \
  packages/interfold-contracts/contracts/lib/ActiveCryptoConfig.sol \
  packages/interfold-contracts/scripts/utils.ts
# 2) fresh write_vk + stage
mkdir -p "$OUT"
for d in "$ART"/secure-8192/micro/default/dkg/*/; do
  n=$(basename "$d"); src="$d$n.json"
  vkt=$(mktemp -d)
  bb write_vk -b "$src" -t noir-recursive -o "$vkt" > "$OUT/../$n.vkw.log" 2>&1
  mkdir -p "$OUT/$n"
  cp "$src" "$OUT/$n/$n.json"; cp "$vkt/vk" "$OUT/$n/$n.vk"; cp "$vkt/vk_hash" "$OUT/$n/$n.vk_hash"
  rm -rf "$vkt"; echo "staged $n"
done
echo "STAGE_TREE_DONE"
# 3) run: E3_R80_STAGE... (use r180 root):
#   E3_R82_STAGE_ROOT=/home/dev/interfold-research/poc/r180/root \
#   cargo test --release -p e3-zk-prover --test c2_micro_prove_tests_r82 -- --nocapture