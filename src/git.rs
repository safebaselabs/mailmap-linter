use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::process::Command;

/// Parse raw git log output into a sorted, deduplicated list of authors.
pub fn parse_git_output(
    author_output: &str,
    committer_output: &str,
) -> Vec<String> {
    let mut authors = HashSet::new();
    for line in author_output.lines() {
        if !line.is_empty() {
            authors.insert(line.to_string());
        }
    }
    for line in committer_output.lines() {
        if !line.is_empty() {
            authors.insert(line.to_string());
        }
    }
    let mut sorted: Vec<String> = authors.into_iter().collect();
    sorted.sort();
    sorted
}

/// Get all git authors and committers from the repository log.
pub fn get_git_authors() -> Result<Vec<String>> {
    let author_output = Command::new("git")
        .args(["log", "--format=%aN <%aE>"])
        .output()?;
    let committer_output = Command::new("git")
        .args(["log", "--format=%cN <%cE>"])
        .output()?;

    if !author_output.status.success() || !committer_output.status.success() {
        return Err(anyhow!("Failed to get git log"));
    }

    let author_str = String::from_utf8(author_output.stdout)?;
    let committer_str = String::from_utf8(committer_output.stdout)?;

    Ok(parse_git_output(&author_str, &committer_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_output_basic() {
        let authors = "Alice Brown <a@g.com>\nBob Smith <b@g.com>\n";
        let committers = "Alice Brown <a@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(
            result,
            vec![
                "Alice Brown <a@g.com>".to_string(),
                "Bob Smith <b@g.com>".to_string(),
            ]
        );
    }

    #[test]
    fn test_parse_git_output_deduplicates() {
        let authors = "Alice <a@g.com>\nAlice <a@g.com>\n";
        let committers = "Alice <a@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result, vec!["Alice <a@g.com>".to_string()]);
    }

    #[test]
    fn test_parse_git_output_empty() {
        let result = parse_git_output("", "");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_git_output_skips_empty_lines() {
        let authors = "Alice <a@g.com>\n\n\nBob <b@g.com>\n";
        let committers = "\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_git_output_sorted() {
        let authors = "Charlie <c@g.com>\nAlice <a@g.com>\n";
        let committers = "Bob <b@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result[0], "Alice <a@g.com>");
        assert_eq!(result[1], "Bob <b@g.com>");
        assert_eq!(result[2], "Charlie <c@g.com>");
    }

    #[test]
    fn test_parse_git_output_merges_authors_and_committers() {
        let authors = "Alice <a@g.com>\n";
        let committers = "Bob <b@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_get_git_authors_in_repo() {
        // This test runs in the actual git repo, so it should work
        let result = get_git_authors();
        assert!(result.is_ok());
        let authors = result.unwrap();
        assert!(!authors.is_empty());
        // Authors should be sorted
        let mut sorted = authors.clone();
        sorted.sort();
        assert_eq!(authors, sorted);
    }

    #[test]
    fn test_parse_git_output_whitespace_handling() {
        let authors = "Alice <a@g.com>\n\n\n";
        let committers = "\n\nBob <b@g.com>\n\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_git_output_case_sensitive() {
        let authors = "alice <a@g.com>\nAlice <a@g.com>\n";
        let committers = "";
        let result = parse_git_output(authors, committers);
        // Should have both lowercase and uppercase (they're different)
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_git_output_special_characters_in_email() {
        let authors = "Alice <alice+tag@g.com>\n";
        let committers = "Bob <bob.name@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"Alice <alice+tag@g.com>".to_string()));
        assert!(result.contains(&"Bob <bob.name@g.com>".to_string()));
    }

    #[test]
    fn test_parse_git_output_only_authors() {
        let authors = "Alice <a@g.com>\nBob <b@g.com>\n";
        let committers = "";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_git_output_only_committers() {
        let authors = "";
        let committers = "Alice <a@g.com>\nBob <b@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_git_output_duplicate_across_authors_and_committers() {
        let authors = "Alice <a@g.com>\nAlice <a@g.com>\n";
        let committers = "Alice <a@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Alice <a@g.com>");
    }

    #[test]
    fn test_parse_git_output_many_entries() {
        let authors = "Alice <a@g.com>\nBob <b@g.com>\nCharlie <c@g.com>\n";
        let committers = "Diana <d@g.com>\nEve <e@g.com>\n";
        let result = parse_git_output(authors, committers);
        assert_eq!(result.len(), 5);
        // Verify it's sorted
        let mut sorted = result.clone();
        sorted.sort();
        assert_eq!(result, sorted);
    }
}
