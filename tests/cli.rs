use assert_cmd::{crate_name, Command};
use assert_fs::{
    fixture::{FileWriteStr, PathChild},
    TempDir,
};
use predicates::prelude::{predicate, PredicateStrExt};

use std::fs;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn belay_in_non_git_dir() -> TestResult {
    let working_dir = TempDir::new()?;

    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::similar(r#"Error: "Failed to find git root""#).trim());

    Ok(())
}

#[test]
fn belay_in_no_ci_dir() -> TestResult {
    let working_dir = TempDir::new()?;
    Command::new("git")
        .arg("init")
        .current_dir(working_dir.path())
        .assert()
        .success();

    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::similar(r#"Error: "Unable to find CI configuration""#).trim());

    Ok(())
}

#[test]
fn belay_in_github_ci_dir_with_restricted_applicability() -> TestResult {
    let working_dir = TempDir::new()?;

    Command::new("git")
        .arg("init")
        .current_dir(working_dir.path())
        .assert()
        .success();
    fs::create_dir_all(working_dir.child(".github").child("workflows").path())?;
    let github_yaml = include_str!("./github_parse_check_on_push_to_branch.yml");
    working_dir
        .child(".github")
        .child("workflows")
        .child("rust.yml")
        .write_str(github_yaml)?;

    // Should run while on master branch
    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::similar(
                r#"Checking 'A Step':
stepping
Success!
"#,
            )
            .normalize(),
        );

    // Should not run while on other branch
    Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg("develop")
        .current_dir(working_dir.path())
        .assert()
        .success();
    // add and commit here to allow our current branch name
    // lookup to work
    Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(working_dir.path())
        .assert()
        .success();
    Command::new("git")
        .arg("-c")
        .arg("user.name='Josh'")
        .arg("-c")
        .arg("user.email='Josh@email.com'")
        .arg("commit")
        .arg("-m")
        .arg("\"test commit\"")
        .current_dir(working_dir.path())
        .assert()
        .success();
    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::similar(""));

    // should run if upstream is set, since the yml specifies it
    // is triggered on pull request
    Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("upstream")
        .arg("test.com")
        .current_dir(working_dir.path())
        .assert()
        .success();
    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::similar(
                r#"Checking 'A Step':
stepping
Success!
"#,
            )
            .normalize(),
        );

    Ok(())
}
#[test]
fn belay_in_github_ci_dir() -> TestResult {
    let working_dir = TempDir::new()?;

    Command::new("git")
        .arg("init")
        .current_dir(working_dir.path())
        .assert()
        .success();
    fs::create_dir_all(working_dir.child(".github").child("workflows").path())?;
    let github_yaml = include_str!("./github_passing_integration_test.yml");
    working_dir
        .child(".github")
        .child("workflows")
        .child("rust.yml")
        .write_str(github_yaml)?;

    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::similar(
                r#"Checking 'Say hello':
hello
Success!
Checking 'Say goodbye':
goodbye
Success!
"#,
            )
            .normalize(),
        );

    Ok(())
}

#[test]
fn belay_in_github_ci_dir_with_multiple_workflows() -> TestResult {
    let working_dir = TempDir::new()?;

    Command::new("git")
        .arg("init")
        .current_dir(working_dir.path())
        .assert()
        .success();
    fs::create_dir_all(working_dir.child(".github").child("workflows").path())?;

    let github_yaml = include_str!("./github_passing_integration_test.yml");
    working_dir
        .child(".github")
        .child("workflows")
        .child("rust.yml")
        .write_str(github_yaml)?;
    let github_yaml = include_str!("./github_failing_integration_test.yml");
    working_dir
        .child(".github")
        .child("workflows")
        .child("rust2.yml")
        .write_str(github_yaml)?;

    // workflows should run in alphabetical order, and scripts which
    // are exactly the same should not be run again
    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::similar(
                r#"Checking 'Say hello':
hello
Success!
Checking 'Say goodbye':
goodbye
Success!
Checking 'tough test':
"#,
            )
            .normalize(),
        )
        .stderr(predicate::str::similar("Error: \"Failed\"").trim());

    Ok(())
}

#[test]
fn belay_in_gitlab_ci_dir() -> TestResult {
    let working_dir = TempDir::new()?;

    Command::new("git")
        .arg("init")
        .current_dir(working_dir.path())
        .assert()
        .success();
    let github_yaml = include_str!("./gitlab_passing_integration_test.yml");
    working_dir.child(".gitlab-ci.yml").write_str(github_yaml)?;

    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::similar(
                r#"Checking 'echo hello':
hello
Success!
"#,
            )
            .normalize(),
        );

    Ok(())
}

#[test]
fn belay_in_github_ci_dir_fails() -> TestResult {
    let working_dir = TempDir::new()?;

    Command::new("git")
        .arg("init")
        .current_dir(working_dir.path())
        .assert()
        .success();
    fs::create_dir_all(working_dir.child(".github").child("workflows").path())?;
    let github_yaml = include_str!("./github_failing_integration_test.yml");
    working_dir
        .child(".github")
        .child("workflows")
        .child("rust.yml")
        .write_str(github_yaml)?;

    Command::cargo_bin(crate_name!())?
        .current_dir(working_dir.path())
        .assert()
        .failure()
        .stdout(
            predicate::str::similar(
                r#"Checking 'Say hello':
hello
Success!
Checking 'tough test':
"#,
            )
            .normalize(),
        )
        .stderr(predicate::str::similar("Error: \"Failed\"").trim());

    Ok(())
}

#[test]
fn belay_hook_push() -> TestResult {
    let working_dir = TempDir::new()?;

    Command::new("git")
        .arg("init")
        .current_dir(working_dir.path())
        .assert()
        .success();

    Command::cargo_bin(crate_name!())?
        .arg("hook")
        .arg("push")
        .current_dir(working_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::similar("Created hook `.git/hooks/pre-push`").trim());

    assert!(working_dir
        .child(".git")
        .child("hooks")
        .child("pre-push")
        .path()
        .exists());

    Ok(())
}
