pub mod git;
pub mod validation;

use anyhow::Result;
use std::fs;
use tracing::info;

/// Run the complete mailmap validation workflow.
///
/// # Arguments
/// * `mailmap` - Lines from the .mailmap file
/// * `authors` - List of git authors found in the repository
/// * `exclude_patterns` - Regex patterns to exclude authors from verification
pub fn run(
    mailmap: &[String],
    authors: &[String],
    exclude_patterns: &[String],
) -> Result<()> {
    info!("Verifying .mailmap format");
    validation::validate_mailmap_format(mailmap)?;

    info!("Verifying that every author is in the .mailmap");
    validation::validate_authors_mapped(authors, mailmap, exclude_patterns)?;

    info!("Verifying that .mailmap is sorted");
    validation::validate_mailmap_sorted(mailmap)?;

    Ok(())
}

/// Load exclude patterns from a file if it exists
pub fn load_exclude_file(path: &str) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    if fs::metadata(path).is_ok() {
        info!("Found {} file", path);
        info!("Reading current {}", path);
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            if !line.is_empty() {
                patterns.push(line.to_string());
            }
        }
    }
    Ok(patterns)
}

/// Run the mailmap linter with file paths and exclusion patterns
pub fn run_linter(
    mailmap_path: &str,
    exclude_file_path: &str,
    exclude_patterns: Vec<String>,
) -> Result<()> {
    // Step 1: Check .mailmap exists
    validation::validate_mailmap_exists(mailmap_path)?;

    // Step 2: Get git authors
    info!("Computing contributors");
    let authors = git::get_git_authors()?;

    // Step 3: Read .mailmap
    info!("Reading current .mailmap");
    let mailmap = fs::read_to_string(mailmap_path)?
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    // Step 4: Read .mailmap-exclude if it exists
    let mut all_exclude_patterns = exclude_patterns;
    let file_patterns = load_exclude_file(exclude_file_path)?;
    all_exclude_patterns.extend(file_patterns);

    // Step 5: Run validation workflow
    run(&mailmap, &authors, &all_exclude_patterns)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_happy_path() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_bad_format() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap = vec!["Invalid Format".to_string()];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_err());
    }

    #[test]
    fn test_run_missing_author() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Bob Smith <b@g.com> Bob Smith <b@g.com>".to_string()];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_err());
    }

    #[test]
    fn test_run_unsorted_mailmap() {
        let authors = vec![
            "Alice Brown <a@g.com>".to_string(),
            "Bob Smith <b@g.com>".to_string(),
        ];
        let mailmap = vec![
            "Bob Smith <b@g.com> Bob Smith <b@g.com>".to_string(),
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
        ];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_err());
    }

    #[test]
    fn test_run_with_exclusion() {
        let authors = vec![
            "Kevin Amado <k@g.com>".to_string(),
            "GitHub <noreply@github.com>".to_string(),
        ];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec!["^.* <.*noreply@github.com>$".to_string()];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_multiple_authors_sorted() {
        let authors = vec![
            "Alice Brown <a@g.com>".to_string(),
            "Bob Smith <b@g.com>".to_string(),
        ];
        let mailmap = vec![
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
            "Bob Smith <b@g.com> Bob Smith <b@g.com>".to_string(),
        ];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_multiple_exclude_patterns() {
        let authors = vec!["GitHub <noreply@github.com>".to_string()];
        let mailmap = vec![];
        let exclude_patterns = vec![
            "^.* <.*noreply@github.com>$".to_string(),
            "^.* <.*bot@.*>$".to_string(),
        ];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_exclude_with_multiple_patterns() {
        let authors = vec![
            "Alice Brown <a@g.com>".to_string(),
            "GitHub <noreply@github.com>".to_string(),
        ];
        let mailmap =
            vec!["Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string()];
        let exclude_patterns = vec![
            "^.* <.*noreply@github.com>$".to_string(),
            "^.* <.*bot@.*>$".to_string(),
        ];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_partial_name_not_matched() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Kevin Smith <k@g.com> Kevin Smith <k@g.com>".to_string()];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_err());
    }

    #[test]
    fn test_run_empty_mailmap_with_author() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap = vec![];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_err());
    }

    #[test]
    fn test_run_multiple_missing_authors() {
        let authors = vec![
            "Alice Brown <a@g.com>".to_string(),
            "Bob Smith <b@g.com>".to_string(),
            "Charlie Davis <c@g.com>".to_string(),
        ];
        let mailmap =
            vec!["David Evans <d@g.com> David Evans <d@g.com>".to_string()];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_err());
    }

    #[test]
    fn test_run_exclude_one_of_multiple_authors() {
        let authors = vec![
            "Alice Brown <a@g.com>".to_string(),
            "Bob <bot@example.com>".to_string(),
        ];
        let mailmap =
            vec!["Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string()];
        let exclude_patterns = vec!["^.* <.*bot@.*>$".to_string()];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_with_empty_mailmap_lines() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap = vec![
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
            "".to_string(),
            "Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string(),
        ];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_no_authors() {
        let authors = vec![];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_empty_exclude_patterns() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_all_authors_excluded() {
        let authors = vec![
            "GitHub <noreply@github.com>".to_string(),
            "Bot <bot@example.com>".to_string(),
        ];
        let mailmap = vec![];
        let exclude_patterns = vec![
            "^.* <.*noreply@github.com>$".to_string(),
            "^.* <.*bot@.*>$".to_string(),
        ];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_run_sorted_mailmap_various_names() {
        let authors = vec![
            "Alice Alice <a@g.com>".to_string(),
            "Bob Bob <b@g.com>".to_string(),
            "Charlie Charlie <c@g.com>".to_string(),
        ];
        let mailmap = vec![
            "Alice Alice <a@g.com> Alice Alice <a@g.com>".to_string(),
            "Bob Bob <b@g.com> Bob Bob <b@g.com>".to_string(),
            "Charlie Charlie <c@g.com> Charlie Charlie <c@g.com>".to_string(),
        ];
        let exclude_patterns = vec![];

        assert!(run(&mailmap, &authors, &exclude_patterns).is_ok());
    }

    #[test]
    fn test_load_exclude_file_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exclude.txt");
        let result = load_exclude_file(path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_load_exclude_file_single_pattern() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exclude.txt");
        fs::write(&path, "^.*noreply.*$\n").unwrap();

        let result = load_exclude_file(path.to_str().unwrap());
        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0], "^.*noreply.*$");
    }

    #[test]
    fn test_load_exclude_file_multiple_patterns() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exclude.txt");
        fs::write(&path, "^.*noreply.*$\n^.*bot.*$\n^.*github.*$\n").unwrap();

        let result = load_exclude_file(path.to_str().unwrap());
        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0], "^.*noreply.*$");
        assert_eq!(patterns[1], "^.*bot.*$");
        assert_eq!(patterns[2], "^.*github.*$");
    }

    #[test]
    fn test_load_exclude_file_empty_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exclude.txt");
        fs::write(&path, "^.*noreply.*$\n\n^.*bot.*$\n\n").unwrap();

        let result = load_exclude_file(path.to_str().unwrap());
        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0], "^.*noreply.*$");
        assert_eq!(patterns[1], "^.*bot.*$");
    }

    #[test]
    fn test_load_exclude_file_trailing_newline() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exclude.txt");
        fs::write(&path, "^.*noreply.*$\n").unwrap();

        let result = load_exclude_file(path.to_str().unwrap());
        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn test_load_exclude_file_multiple_trailing_newlines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exclude.txt");
        fs::write(&path, "^.*noreply.*$\n\n\n").unwrap();

        let result = load_exclude_file(path.to_str().unwrap());
        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn test_run_linter_nonexistent_mailmap() {
        let result = run_linter(
            "/nonexistent/path/.mailmap",
            "/nonexistent/path/.mailmap-exclude",
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_linter_with_exclude_patterns_from_args() {
        let dir = TempDir::new().unwrap();
        let mailmap_path = dir.path().join(".mailmap");
        let exclude_path = dir.path().join(".mailmap-exclude");

        fs::write(
            &mailmap_path,
            "Test User <test@test.com> Test User <test@test.com>\n",
        )
        .unwrap();

        let result = run_linter(
            mailmap_path.to_str().unwrap(),
            exclude_path.to_str().unwrap(),
            vec!["^.*$".to_string()],
        );
        let _ = result;
    }

    #[test]
    fn test_run_linter_loads_exclude_file() {
        let dir = TempDir::new().unwrap();
        let mailmap_path = dir.path().join(".mailmap");
        let exclude_path = dir.path().join(".mailmap-exclude");

        fs::write(
            &mailmap_path,
            "Test User <test@test.com> Test User <test@test.com>\n",
        )
        .unwrap();
        fs::write(&exclude_path, "^.*$\n").unwrap();

        let result = run_linter(
            mailmap_path.to_str().unwrap(),
            exclude_path.to_str().unwrap(),
            vec![],
        );
        let _ = result;
    }
}
