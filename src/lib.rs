//! Patience diff algorithm implementation.
//!
//! Patience diff matches unique anchor lines first, then recursively diffs
//! the regions between anchors. Often produces more human-readable results
//! than standard LCS diff for code with moved blocks.

/// A single line in a patience diff result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLine {
    Context(String),
    Add(String),
    Remove(String),
}

/// A hunk produced by patience diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatienceHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

/// Result of a patience diff between two texts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatienceDiff {
    pub hunks: Vec<PatienceHunk>,
}

impl PatienceDiff {
    /// Compute patience diff between two texts.
    pub fn diff(old: &str, new: &str) -> Self {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();
        let lines = patience_diff_lines(&old_lines, &new_lines);

        // Group into hunks
        let hunks = group_into_hunks(&lines);
        PatienceDiff { hunks }
    }

    /// Render as unified diff format.
    pub fn to_unified(&self, old_label: &str, new_label: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("--- {old_label}\n"));
        out.push_str(&format!("+++ {new_label}\n"));
        for hunk in &self.hunks {
            out.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
            ));
            for line in &hunk.lines {
                match line {
                    DiffLine::Context(t) => out.push_str(&format!(" {t}\n")),
                    DiffLine::Add(t) => out.push_str(&format!("+{t}\n")),
                    DiffLine::Remove(t) => out.push_str(&format!("-{t}\n")),
                }
            }
        }
        out
    }

    /// Count added lines.
    pub fn added(&self) -> usize {
        self.hunks.iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, DiffLine::Add(_)))
            .count()
    }

    /// Count removed lines.
    pub fn removed(&self) -> usize {
        self.hunks.iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, DiffLine::Remove(_)))
            .count()
    }
}

/// Core patience diff algorithm.
fn patience_diff_lines(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    // Step 1: Find unique lines that appear exactly once in both old and new
    let old_counts = line_counts(old);
    let new_counts = line_counts(new);

    let mut anchors: Vec<(usize, usize)> = Vec::new();
    for (oi, line) in old.iter().enumerate() {
        if old_counts.get(*line) == Some(&1) && new_counts.get(*line) == Some(&1) {
            if let Some(ni) = new.iter().position(|l| l == line) {
                anchors.push((oi, ni));
            }
        }
    }

    // Step 2: Find longest increasing subsequence of anchor positions in new
    anchors.sort_by_key(|(_, ni)| *ni);
    let lis = longest_increasing_subsequence_oi(&anchors);
    anchors = lis;

    // Step 3: Recursively diff between anchors using simple LCS
    let mut result = Vec::new();

    // Before first anchor
    let first_old = anchors.first().map(|(oi, _)| *oi).unwrap_or(old.len());
    let first_new = anchors.first().map(|(_, ni)| *ni).unwrap_or(new.len());
    result.extend(simple_lcs_diff(&old[..first_old], &new[..first_new]));

    // Between anchors
    for i in 0..anchors.len() {
        let (oi, ni) = anchors[i];
        result.push(DiffLine::Context(old[oi].to_string()));

        let next_oi = if i + 1 < anchors.len() { anchors[i + 1].0 } else { old.len() };
        let next_ni = if i + 1 < anchors.len() { anchors[i + 1].1 } else { new.len() };

        result.extend(simple_lcs_diff(
            &old[oi + 1..next_oi],
            &new[ni + 1..next_ni],
        ));
    }

    result
}

fn line_counts<'a>(lines: &[&'a str]) -> std::collections::HashMap<&'a str, usize> {
    let mut counts = std::collections::HashMap::new();
    for line in lines {
        *counts.entry(*line).or_insert(0) += 1;
    }
    counts
}

fn longest_increasing_subsequence_oi(anchors: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if anchors.is_empty() {
        return Vec::new();
    }
    let n = anchors.len();
    let mut dp = vec![1usize; n];
    let mut parent = vec![None; n];

    for i in 1..n {
        for j in 0..i {
            if anchors[j].0 < anchors[i].0 && anchors[j].1 < anchors[i].1 && dp[j] + 1 > dp[i] {
                dp[i] = dp[j] + 1;
                parent[i] = Some(j);
            }
        }
    }

    let mut best_end = 0;
    for i in 1..n {
        if dp[i] > dp[best_end] {
            best_end = i;
        }
    }

    let mut result = Vec::new();
    let mut current = Some(best_end);
    while let Some(idx) = current {
        result.push(anchors[idx]);
        current = parent[idx];
    }
    result.reverse();
    result
}

fn simple_lcs_diff(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    if old.is_empty() && new.is_empty() {
        return Vec::new();
    }
    if old.is_empty() {
        return new.iter().map(|l| DiffLine::Add(l.to_string())).collect();
    }
    if new.is_empty() {
        return old.iter().map(|l| DiffLine::Remove(l.to_string())).collect();
    }

    let lcs = longest_common_subsequence(old, new);
    let mut result = Vec::new();
    let mut oi = 0usize;
    let mut ni = 0usize;
    let mut li = 0usize;

    while oi < old.len() || ni < new.len() {
        if li < lcs.len() && oi < old.len() && ni < new.len()
            && old[oi] == lcs[li] && new[ni] == lcs[li]
        {
            result.push(DiffLine::Context(old[oi].to_string()));
            oi += 1; ni += 1; li += 1;
        } else if oi < old.len() && (li >= lcs.len() || old[oi] != lcs[li]) {
            result.push(DiffLine::Remove(old[oi].to_string()));
            oi += 1;
        } else if ni < new.len() && (li >= lcs.len() || new[ni] != lcs[li]) {
            result.push(DiffLine::Add(new[ni].to_string()));
            ni += 1;
        } else {
            break;
        }
    }
    result
}

fn longest_common_subsequence<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();
    if m > 300 || n > 300 {
        return a.iter().filter(|l| b.contains(l)).cloned().collect();
    }
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }
    let mut result = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1; j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

fn group_into_hunks(lines: &[DiffLine]) -> Vec<PatienceHunk> {
    if lines.is_empty() {
        return Vec::new();
    }
    let removes = lines.iter().filter(|l| matches!(l, DiffLine::Remove(_))).count();
    let adds = lines.iter().filter(|l| matches!(l, DiffLine::Add(_))).count();
    let ctx = lines.iter().filter(|l| matches!(l, DiffLine::Context(_))).count();

    vec![PatienceHunk {
        old_start: 1,
        old_count: removes + ctx,
        new_start: 1,
        new_count: adds + ctx,
        lines: lines.to_vec(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_diff() {
        let diff = PatienceDiff::diff("a\nb\nc", "a\nb\nc");
        assert!(diff.hunks.is_empty() || diff.hunks.iter().all(|h|
            h.lines.iter().all(|l| matches!(l, DiffLine::Context(_)))));
    }

    #[test]
    fn test_simple_change() {
        let diff = PatienceDiff::diff("a\nb\nc", "a\nx\nc");
        assert!(diff.added() > 0 || diff.removed() > 0);
    }

    #[test]
    fn test_add_line() {
        let diff = PatienceDiff::diff("a\nc", "a\nb\nc");
        assert!(diff.added() >= 1);
    }

    #[test]
    fn test_render_unified() {
        let diff = PatienceDiff::diff("hello\nworld", "hello\nearth");
        let unified = diff.to_unified("a.txt", "b.txt");
        assert!(unified.contains("--- a.txt"));
        assert!(unified.contains("+++ b.txt"));
    }
}

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}
