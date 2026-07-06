#!/usr/bin/env bash
#
# check-palette-overlap.sh — verify NO hex is shared across the 5 brand-family
# palettes. Run after editing any tokens.css or any family source SVG.
#
# Reads canonical source files per family; checks pairwise for shared hexes
# (case-insensitive #xxxxxx).
#
# Pure bash + grep/sort. No Python. Safe for CI.
#
# Output: per-family hex counts + pairwise matrix + exit 0 on clean / exit 1 on fail.

set -eo pipefail

PHENOTYPE_ROOT="${PHENOTYPE_ROOT:-/Users/kooshapari/CodeProjects/Phenotype/repos}"
HEX_RE='#[0-9a-fA-F]{6}'

# Canvas-default colors that don't count as brand tokens (universal SVG defaults,
# not part of any family's intentional palette). Filtered out of overlap checks.
RGV_IGNORES=("#ffffff" "#000000" "#000")

# Per-family source file list. Use bash arrays of paths (relative to PHENOTYPE_ROOT).
# Backbone-2 ships in TWO source files (sharecli + substrate); we union both.
bb2_sharecli="$PHENOTYPE_ROOT/sharecli/.claude/worktrees/sharecli-iconset/assets/brand/sharecli-icon.svg"
bb2_substrate="$PHENOTYPE_ROOT/substrate/.claude/worktrees/substrate-iconset/assets/brand/substrate-icon.svg"
lc_session="$PHENOTYPE_ROOT/SessionLedger/.claude/worktrees/sessionledger-iconset/assets/brand/sessionledger-icon.svg"
tf_forgecode="$PHENOTYPE_ROOT/forgecode/.claude/worktrees/forgecode-iconset/assets/brand/forgecode-icon.svg"
tr_tracera="$PHENOTYPE_ROOT/Tracera/assets/brand/icon.svg"
mv_melosviz="$PHENOTYPE_ROOT/melosviz/desktop/assets/brand/tokens.css"

declare -a FAMILY_NAMES=("Backbone-2" "Lab-Coat" "Terminal-Forge" "Tracera" "MelosViz")
declare -a FAMILY_COUNTS=(0 0 0 0 0)

# Build hex stream per family (lowercased, sorted, unique).
extract() {
  local file="$1"
  [ -f "$file" ] || { echo ""; return; }
  grep -ohE "$HEX_RE" "$file" 2>/dev/null | tr 'A-Z' 'a-z' | sort -u
}

bb2_hex="$( ( extract "$bb2_sharecli"; extract "$bb2_substrate" ) | sort -u | tr '\n' ' ' )"
lc_hex="$( extract "$lc_session" | tr '\n' ' ' )"
tf_hex="$( extract "$tf_forgecode" | tr '\n' ' ' )"
tr_hex="$( extract "$tr_tracera" | tr '\n' ' ' )"
mv_hex="$( extract "$mv_melosviz" | tr '\n' ' ' )"

# Trim trailing spaces.
bb2_hex="${bb2_hex% }"; lc_hex="${lc_hex% }"
tf_hex="${tf_hex% }"; tr_hex="${tr_hex% }"; mv_hex="${mv_hex% }"

# Count unique hexes per family.
FAMILY_HEXES=("$bb2_hex" "$lc_hex" "$tf_hex" "$tr_hex" "$mv_hex")

for ((i = 0; i < ${#FAMILY_NAMES[@]}; i++)); do
  set -- ${FAMILY_HEXES[$i]}
  printf "  %-15s  %3d unique hexes\n" "${FAMILY_NAMES[$i]}" "$#"
done

# Pairwise overlap.
echo
echo "pairwise overlap matrix (canvas-default #ffffff/#000000 ignored):"

pass=0
fail=0
for ((i = 0; i < ${#FAMILY_NAMES[@]}; i++)); do
  for ((j = i + 1; j < ${#FAMILY_NAMES[@]}; j++)); do
    a="${FAMILY_NAMES[$i]}"
    b="${FAMILY_NAMES[$j]}"
    hex_a="${FAMILY_HEXES[$i]}"
    hex_b="${FAMILY_HEXES[$j]}"
    overlap=""
    for h in $hex_a; do
      # Filter canvas-default colors (universal SVG fallback).
      skip=0
      for ig in "${RGV_IGNORES[@]}"; do
        if [ "$h" = "$ig" ]; then skip=1; break; fi
      done
      [ "$skip" -eq 1 ] && continue
      # word-boundary match (hexes are space-separated)
      if [[ " $hex_b " == *" $h "* ]]; then
        if [ -z "$overlap" ]; then overlap="$h"; else overlap="$overlap $h"; fi
      fi
    done
    if [ -z "$overlap" ]; then
      printf "  %-15s vs %-15s  PASS\n" "$a" "$b"
      pass=$((pass + 1))
    else
      printf "  %-15s vs %-15s  FAIL  (%s)\n" "$a" "$b" "$overlap"
      fail=$((fail + 1))
    fi
  done
done

echo
printf "summary: %d pass, %d fail (10 pairs total)\n" "$pass" "$fail"
[ "$fail" -eq 0 ] || exit 1
exit 0