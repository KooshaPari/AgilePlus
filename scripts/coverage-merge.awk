# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Merge llvm-cov report rows into one workspace summary.
#
# Invoked by scripts/coverage-complete.sh as:
#   awk -f scripts/coverage-merge.awk -v ancestor=AgilePlus chunks.txt
#
# Input: the concatenated `llvm-cov report` output of several object chunks.
# Output: a FILE_ROW section (one line per workspace file: path, lines,
# covered), then a CRATE_TABLE section, then a WORKSPACE_TOTAL line.
#
# coverage-complete.sh's completeness assertion consumes FILE_ROW: every
# production .rs file must appear there or the run fails, so this section must
# never be dropped or truncated silently.
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

# Skip the report header and llvm-cov's own TOTAL row: neither is a file, and
# letting TOTAL reach the "unresolvable row" warning below would be misleading.
NF >= 13 && ($1 == "Filename" || $1 == "TOTAL") { next }

NF >= 13 {
    f = $1
    sub(/^\/+/, "", f)
    idx = index(f, ancestor "/")
    if (idx > 0) f = substr(f, idx + length(ancestor) + 1)

    # Workspace-owned sources only. llvm-cov prints every path with the
    # invocation's longest common prefix stripped, so against a mixed object set
    # that prefix can be the `crates/` component itself: a whole crate then
    # arrives bare (`agileplus-subcmds/src/lib.rs`) and was dropped here,
    # silently losing the crate (subcmds, sqlite, p2p, triage, ...). If the bare
    # path resolves under crates/, restore the prefix BEFORE the check. getline
    # returning >= 0 is the POSIX-awk exists test (plain awk has no stat()).
    if (f !~ /^(crates|desktop|libs|agileplus-agents)\//) {
        cand = "crates/" f
        if ((getline _probe < cand) >= 0) { close(cand); f = cand }
    }
    # Never drop a row silently again: a path that still cannot be resolved is
    # surfaced (stderr), not swallowed - silent row loss is what hid the
    # missing files before.
    if (f !~ /^(crates|desktop|libs|agileplus-agents)\//) {
        if (!(f in warned)) {
            warned[f] = 1
            print "WARNING: dropping unresolvable coverage row: " f > "/dev/stderr"
        }
        next
    }
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
    # FILE_ROW first, so a reader (and the completeness assertion) can diff
    # the per-file rows against the production file list on disk.
    for (f in seen) {
        printf "FILE_ROW %s %d %d\n", f, lines[f], cov[f]
    }
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
