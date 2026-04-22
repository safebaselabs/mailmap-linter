use anyhow::{Result, anyhow};
use regex::Regex;
use std::fs;
use tracing::{error, info};

pub fn validate_mailmap_exists(path: &str) -> Result<()> {
    info!("Verifying if .mailmap exists");
    if fs::metadata(path).is_err() {
        return Err(anyhow!(
            "Please add a .mailmap file in the root of the repo"
        ));
    }
    Ok(())
}

pub fn validate_mailmap_format(mailmap: &[String]) -> Result<()> {
    let format = r"^\p{L}[\p{L}\-]+ \p{L}[\p{L}\-]+ <.*> (.*?) <.*?>$";
    let regex = Regex::new(format)?;

    for (line_number, mapping) in mailmap.iter().enumerate() {
        if !mapping.is_empty() && !regex.is_match(mapping) {
            error!("Invalid mailmap format. Expected: {}", format);
            error!("Line {}: {}", line_number + 1, mapping);
            return Err(anyhow!("Invalid mailmap format"));
        }
    }

    Ok(())
}

pub fn validate_mailmap_sorted(mailmap: &[String]) -> Result<()> {
    let mut sorted_mailmap: Vec<&String> =
        mailmap.iter().filter(|line| !line.is_empty()).collect();
    sorted_mailmap.sort();

    let original: Vec<&String> =
        mailmap.iter().filter(|line| !line.is_empty()).collect();

    if sorted_mailmap == original {
        info!("Mailmap is correctly sorted");
        Ok(())
    } else {
        error!("Please sort the .mailmap with: $ LC_ALL=C sort .mailmap");
        Err(anyhow!("Mailmap is not sorted"))
    }
}

pub fn validate_authors_mapped(
    authors: &[String],
    mailmap: &[String],
    exclude_patterns: &[String],
) -> Result<()> {
    let mut success = true;
    let mut missing_authors = Vec::new();

    for author in authors {
        info!("Verifying: {}", author);

        if is_author_excluded(author, exclude_patterns)? {
            info!("Excluded: {}", author);
            continue;
        }

        let found = mailmap.iter().any(|mapping| mapping.contains(author));

        if !found {
            error!("Please add a .mailmap entry for: {}", author);
            missing_authors.push(author.clone());
            success = false;
        }
    }

    if !success {
        return Err(anyhow!(
            "Missing authors in mailmap: {:?}",
            missing_authors
        ));
    }

    Ok(())
}

pub fn is_author_excluded(
    author: &str,
    exclude_patterns: &[String],
) -> Result<bool> {
    for pattern in exclude_patterns {
        let regex = Regex::new(pattern)?;
        if regex.is_match(author) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_mailmap_exists_missing() {
        assert!(validate_mailmap_exists("/nonexistent/path/.mailmap").is_err());
    }

    #[test]
    fn test_validate_mailmap_exists_present() {
        // Use a file that exists in the repo
        assert!(validate_mailmap_exists("Cargo.toml").is_ok());
    }

    #[test]
    fn test_mailmap_format_valid() {
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        assert!(validate_mailmap_format(&mailmap).is_ok());
    }

    #[test]
    fn test_mailmap_format_lowercase_start_now_allowed() {
        let mailmap =
            vec!["kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        // Lowercase letters are now allowed as they are valid Unicode letters
        assert!(validate_mailmap_format(&mailmap).is_ok());
    }

    #[test]
    fn test_mailmap_format_invalid_single_name() {
        let mailmap = vec!["Kevin <k@g.com> Kevin Amado <k@g.com>".to_string()];
        assert!(validate_mailmap_format(&mailmap).is_err());
    }

    #[test]
    fn test_mailmap_format_invalid_missing_email() {
        let mailmap = vec!["Kevin Amado Kevin Amado <k@g.com>".to_string()];
        assert!(validate_mailmap_format(&mailmap).is_err());
    }

    #[test]
    fn test_mailmap_format_empty_line() {
        let mailmap = vec![
            "Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string(),
            "".to_string(),
        ];
        assert!(validate_mailmap_format(&mailmap).is_ok());
    }

    #[test]
    fn test_mailmap_format_accented_characters() {
        let mailmap =
            vec!["José García <j@g.com> José García <j@g.com>".to_string()];
        // Unicode letters including accented characters are now supported
        assert!(validate_mailmap_format(&mailmap).is_ok());
    }

    #[test]
    fn test_mailmap_format_hyphenated_names() {
        let mailmap = vec![
            "Jean-Luc Dupont <j@g.com> Jean-Luc Dupont <j@g.com>".to_string(),
        ];
        // Hyphens in names are now supported
        assert!(validate_mailmap_format(&mailmap).is_ok());
    }

    #[test]
    fn test_mailmap_sorted_already_sorted() {
        let mailmap = vec![
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
            "Bob Smith <b@g.com> Bob Smith <b@g.com>".to_string(),
            "Charlie Davis <c@g.com> Charlie Davis <c@g.com>".to_string(),
        ];
        assert!(validate_mailmap_sorted(&mailmap).is_ok());
    }

    #[test]
    fn test_mailmap_sorted_unsorted() {
        let mailmap = vec![
            "Charlie Davis <c@g.com> Charlie Davis <c@g.com>".to_string(),
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
            "Bob Smith <b@g.com> Bob Smith <b@g.com>".to_string(),
        ];
        assert!(validate_mailmap_sorted(&mailmap).is_err());
    }

    #[test]
    fn test_exclude_regex_matches() {
        let exclude_patterns = vec!["^.* <.*noreply@github.com>$".to_string()];
        let result = is_author_excluded(
            "GitHub <noreply@github.com>",
            &exclude_patterns,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_exclude_regex_no_match() {
        let exclude_patterns = vec!["^.* <.*noreply@github.com>$".to_string()];
        let result =
            is_author_excluded("Kevin Amado <k@g.com>", &exclude_patterns);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_exclude_multiple_patterns() {
        let exclude_patterns = vec![
            "^.* <.*noreply@github.com>$".to_string(),
            "^.* <.*no-reply@.*>$".to_string(),
        ];
        let result1 = is_author_excluded(
            "GitHub <noreply@github.com>",
            &exclude_patterns,
        );
        let result2 =
            is_author_excluded("Bot <no-reply@example.com>", &exclude_patterns);
        let result3 =
            is_author_excluded("Kevin Amado <k@g.com>", &exclude_patterns);

        assert!(result1.is_ok());
        assert!(result1.unwrap());
        assert!(result2.is_ok());
        assert!(result2.unwrap());
        assert!(result3.is_ok());
        assert!(!result3.unwrap());
    }

    #[test]
    fn test_validate_authors_mapped_all_found() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec![];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_authors_mapped_missing() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Bob Smith <b@g.com> Bob Smith <b@g.com>".to_string()];
        let exclude_patterns = vec![];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_err()
        );
    }

    #[test]
    fn test_validate_authors_mapped_multiple_missing() {
        let authors = vec![
            "Kevin Amado <k@g.com>".to_string(),
            "Alice Brown <a@g.com>".to_string(),
            "Bob Smith <b@g.com>".to_string(),
        ];
        let mailmap = vec!["Charlie <c@g.com> Charlie <c@g.com>".to_string()];
        let exclude_patterns = vec![];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_err()
        );
    }

    #[test]
    fn test_validate_authors_mapped_with_exclusion() {
        let authors = vec![
            "Kevin Amado <k@g.com>".to_string(),
            "GitHub <noreply@github.com>".to_string(),
        ];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec!["^.* <.*noreply@github.com>$".to_string()];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_authors_mapped_partial_exclusion() {
        let authors = vec![
            "Kevin Amado <k@g.com>".to_string(),
            "Alice Brown <a@g.com>".to_string(),
            "GitHub <noreply@github.com>".to_string(),
        ];
        let mailmap = vec![
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
            "Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string(),
        ];
        let exclude_patterns = vec!["^.* <.*noreply@github.com>$".to_string()];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_authors_mapped_empty_authors() {
        let authors = vec![];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec![];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_authors_mapped_empty_mailmap() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap = vec![];
        let exclude_patterns = vec![];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_err()
        );
    }

    #[test]
    fn test_validate_authors_mapped_partial_match() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap = vec!["Kevin <k@g.com> Kevin <k@g.com>".to_string()];
        let exclude_patterns = vec![];
        // Should fail because "Kevin Amado <k@g.com>" is not in the mailmap
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_err()
        );
    }

    #[test]
    fn test_validate_authors_mapped_contains_match() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        // Should pass because the mailmap contains the author string
        let exclude_patterns = vec![];
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_ok()
        );
    }

    #[test]
    fn test_exclude_regex_invalid_pattern() {
        let exclude_patterns = vec!["[invalid".to_string()];
        let result =
            is_author_excluded("Kevin Amado <k@g.com>", &exclude_patterns);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_mailmap_format_multiple_valid() {
        let mailmap = vec![
            "kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string(),
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
        ];
        // Both are now valid as lowercase letters are allowed
        assert!(validate_mailmap_format(&mailmap).is_ok());
    }

    #[test]
    fn test_validate_mailmap_sorted_with_empty_lines() {
        let mailmap = vec![
            "Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string(),
            "".to_string(),
            "Bob Smith <b@g.com> Bob Smith <b@g.com>".to_string(),
        ];
        assert!(validate_mailmap_sorted(&mailmap).is_ok());
    }

    #[test]
    fn test_validate_mailmap_sorted_single_entry() {
        let mailmap =
            vec!["Alice Brown <a@g.com> Alice Brown <a@g.com>".to_string()];
        assert!(validate_mailmap_sorted(&mailmap).is_ok());
    }

    #[test]
    fn test_validate_mailmap_format_complex_valid() {
        // Test with a more complex but valid mailmap entry
        let mailmap = vec![
            "Test User <test@test.com> Test User <test@test.com>".to_string(),
        ];
        assert!(validate_mailmap_format(&mailmap).is_ok());
    }

    #[test]
    fn test_authors_mapped_with_invalid_exclude_regex() {
        let authors = vec!["Kevin Amado <k@g.com>".to_string()];
        let mailmap =
            vec!["Kevin Amado <k@g.com> Kevin Amado <k@g.com>".to_string()];
        let exclude_patterns = vec!["[invalid".to_string()];
        // Should fail due to invalid regex in exclude patterns
        assert!(
            validate_authors_mapped(&authors, &mailmap, &exclude_patterns)
                .is_err()
        );
    }
}
