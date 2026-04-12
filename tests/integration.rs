use assert_cmd::Command;
use std::fs;
use std::process;
use tempfile::TempDir;

fn setup_git_repo(dir: &TempDir, mailmap: Option<&str>, exclude: Option<&str>) {
    let path = dir.path();

    // Initialize git repo
    let output = process::Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Configure git user
    process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();
    process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();
    // Disable GPG signing for test commits
    process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(path)
        .output()
        .unwrap();

    // Write .mailmap if provided
    if let Some(content) = mailmap {
        fs::write(path.join(".mailmap"), content).unwrap();
    }

    // Write .mailmap-exclude if provided
    if let Some(content) = exclude {
        fs::write(path.join(".mailmap-exclude"), content).unwrap();
    }

    // Create a dummy file and commit
    fs::write(path.join("dummy.txt"), "hello").unwrap();
    process::Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();
    process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(path)
        .output()
        .unwrap();
}

#[test]
fn test_no_mailmap_file() {
    let dir = TempDir::new().unwrap();
    setup_git_repo(&dir, None, None);
    // Remove .mailmap if setup created it
    let _ = fs::remove_file(dir.path().join(".mailmap"));

    Command::cargo_bin("mailmap-linter")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_valid_mailmap() {
    let dir = TempDir::new().unwrap();
    setup_git_repo(
        &dir,
        Some("Test User <test@test.com> Test User <test@test.com>\n"),
        None,
    );

    Command::cargo_bin("mailmap-linter")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn test_invalid_format() {
    let dir = TempDir::new().unwrap();
    setup_git_repo(&dir, Some("invalid format\n"), None);

    Command::cargo_bin("mailmap-linter")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_missing_author() {
    let dir = TempDir::new().unwrap();
    setup_git_repo(
        &dir,
        Some("Other Person <other@test.com> Other Person <other@test.com>\n"),
        None,
    );

    Command::cargo_bin("mailmap-linter")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_unsorted_mailmap() {
    let dir = TempDir::new().unwrap();
    let mailmap = "Test User <test@test.com> Test User <test@test.com>\nAlice Brown <a@g.com> Alice Brown <a@g.com>\n";
    setup_git_repo(&dir, Some(mailmap), None);

    Command::cargo_bin("mailmap-linter")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_exclude_flag() {
    let dir = TempDir::new().unwrap();
    // Don't add the commit author to mailmap; exclude them via flag
    setup_git_repo(&dir, Some(""), None);

    Command::cargo_bin("mailmap-linter")
        .unwrap()
        .current_dir(dir.path())
        .args(["--exclude", "^.*$"])
        .assert()
        .success();
}

#[test]
fn test_mailmap_exclude_file() {
    let dir = TempDir::new().unwrap();
    setup_git_repo(&dir, Some(""), Some("^.*$\n"));

    Command::cargo_bin("mailmap-linter")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .success();
}
