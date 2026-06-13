# Patience Diff

**An implementation of the Patience diff algorithm** that matches unique anchor lines first, then recursively diffs the regions between them. This produces more human-readable diffs than standard LCS for code with moved blocks, added imports, or brace-shifted functions.

## Why It Matters

Standard diff algorithms (Myers, LCS) optimize for the *minimum* number of changes. But the minimum edit script isn't always the most *readable* one. Consider: if you add an import statement at the top of a file, a standard diff might realign everything, showing the entire file as changed. Patience diff handles this better.

**The key insight:** In real-world code, certain lines are unique and stable — function signatures, class definitions, return statements. These serve as natural "anchors" that align the diff. By matching anchors first and recursively diffing the gaps, Patience diff produces diffs that follow the code's logical structure.

**Named after the Patience card game** (also called "Idiot's Delight"), where cards are sorted by repeatedly finding and extracting the longest increasing subsequence. The algorithm was created by Bram Cohen (BitTorrent creator) and is used by **Git's `--patience` flag** and **Bazaar** (Canonical's VCS).

**When to use Patience vs Myers:**
- **Patience** — Better for code review: moved blocks, added imports, nested changes
- **Myers** — Better for minimal output: small diffs, automated processing, CI pipelines

## How It Works

The algorithm proceeds in four phases:

**1. Find unique common lines:** Scan both old and new text for lines that appear *exactly once* in each. These are the anchor candidates — they're unambiguous matches. A line appearing multiple times in either text is too ambiguous to anchor reliably.

**2. Longest Increasing Subsequence (LIS):** Among the anchor pairs (old_index, new_index), find the longest subsequence where both indices are monotonically increasing. This is the classic Patience sorting problem — O(n²) with dynamic programming. The LIS gives us the set of anchors that are consistent with both orderings.

**3. Recursive diffing:** Between consecutive anchors, recursively apply the same algorithm. If no unique lines exist in a region, fall back to standard LCS diff. This recursion terminates because each level operates on strictly smaller inputs.

**4. LCS fallback:** For regions with no unique anchors (e.g., blocks of similar-looking lines), a standard O(N×M) LCS computation produces the local diff. For large regions (>300 lines), a simple intersection filter avoids the O(N×M) blowup.

**Why it produces better diffs:** By anchoring on unique lines like `function foo() {`, the algorithm naturally aligns function boundaries. If you add a new function in the middle of a file, the diff shows just that addition, rather than cascading changes as the LCS tries to find the cheapest (but not most readable) alignment.

## Quick Start

```rust
use diff_patience::PatienceDiff;

let old = "import os\nimport sys\n\ndef main():\n    pass";
let new = "import os\nimport sys\nimport json\n\ndef main():\n    pass";

let diff = PatienceDiff::diff(old, new);
println!("Added: {}, Removed: {}", diff.added(), diff.removed());

let unified = diff.to_unified("old.py", "new.py");
print!("{}", unified);
```

## API

### `PatienceDiff`
- `diff(old: &str, new: &str) -> Self` — Compute patience diff
- `to_unified(old_label, new_label) -> String` — Render as unified diff format
- `added() -> usize` — Count of added lines
- `removed() -> usize` — Count of removed lines
- `hunks: Vec<PatienceHunk>` — The diff hunks

### `PatienceHunk`
- `old_start, old_count, new_start, new_count: usize` — Line positions
- `lines: Vec<DiffLine>` — Line-level changes

### `DiffLine`
- `Context(String)` — Unchanged line
- `Add(String)` — Added line
- `Remove(String)` — Removed line

## Architecture Notes

Part of SuperInstance's text processing toolkit alongside Myers diff, diff-patch, and three-way merge. Patience diff is preferred for human-facing diffs (code review, documentation) while Myers diff is used for automated processing where minimal output matters.

See the full architecture: [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md)

## License

MIT
