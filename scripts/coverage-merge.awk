# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Merge llvm-cov report rows into one workspace summary.
#
# Invoked by scripts/coverage-complete.sh as:
#   awk -f scripts/coverage-merge.awk -v ancestor=AgilePlus chunks.txt
#
# Input: the concatenated `llvm-cov report` output of several object chunks.
# Output: a CRATE_TABLE section, then a WORKSPACE_TOTAL line.
#
# Two things this has to get right:
#
#  1. Paths. `llvm-cov` prints every path with the longest common prefix of all
#     reported paths stripped, so the workspace root can arrive truncated to any
#     depth - `CodeProjects/Phenotype/repos/AgilePlus/...`, `AgilePlus/...`, or
#     absolute, depending on which objects were passed. Anchoring on an absolute
#     prefix silently matched nothing. Anchor on the repository directory name.
#
#  2. Deduplication. A file can appear in more than one compilation variant of
#     its crate (different feature sets), and the same file is also linked into
#     the workspace binaries. A file belongs to exactly one crate, so the largest
#     line count is its real line count and the largest covered count is its real
#     coverage. Summing would double-count.

NF >= 13 && $1 == "Filename" { next }

NF >= 13 {
    f = $1
    sub(/^\/+/, "", f)
    idx = index(f, ancestor "/")
    if (idx > 0) f = substr(f, idx + length(ancestor) + 1)

    # Workspace-owned sources only.
    if (f !~ /^(crates|desktop|libs|agileplus-agents)\//) next
    # Test/example/bench sources are excluded by -ignore-filename-regex; repeat
    # it here because a printed path may already be relative. The slash inside
    # the bracket must be escaped: awk's lexer ends a regex token at the first
    # unescaped `/` even inside a bracket expression.
    if (f ~ /^crates\/[^\/]+\/tests\//) next

    if ($8 > lines[f]) lines[f] = $8
    covered = $8 - $9
    if (covered > cov[f]) cov[f] = covered
    seen[f] = 1
    next
}

END {
    for (f in seen) {
        total += lines[f]
        covered_total += cov[f]
        n++
        split(f, a, "/")
        # crates/<crate>, agileplus-agents/crates/<crate>, desktop/<app>,
        # libs/<lib>, tests/<suite>
        if (a[1] == "agileplus-agents" && a[2] == "crates") crate = a[1] "/" a[3]
        else crate = a[1] "/" a[2]
        clines[crate] += lines[f]
        ccov[crate] += cov[f]
        cfile[crate]++
    }

    printf "CRATE_TABLE\n"
    for (c in clines)
        printf "%-44s files=%-4d lines=%-7d covered=%-7d %6.2f%%\n",
            c, cfile[c], clines[c], ccov[c],
            (clines[c] ? ccov[c] * 100 / clines[c] : 0)
    printf "TOTAL\n"
    printf "WORKSPACE_TOTAL %d %d %d\n", n, total, covered_total
}
