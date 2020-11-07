use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

// // // // // // Utility Functions // // // // // // 

fn rm_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let error_msg = format!("failed to remove {}", path);

    let cmd = Command::new("rm")
        .arg("-f")
        .arg(path)
        .output()
        .expect(&error_msg);
    cmd.assert().success();
    Ok(())
}


// // // // // // Parameter Store // // // // // // 

#[test]
fn test_ps_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("check").arg("-f").arg("./tests/param_store_mem.toml");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("World"));

    Ok(())
}

#[test]
fn test_ps_query() -> Result<(), Box<dyn std::error::Error>> {

    rm_file(&"tests/ps.db")?;

    // Check for an empty cache
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("query").arg("-f").arg("./tests/param_store_db.toml");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(""));

    // Get some data
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("check").arg("-f").arg("./tests/param_store_db.toml");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("World"));
    
    // Check for an data in cache
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("query").arg("-f").arg("./tests/param_store_db.toml");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("World"));

    rm_file(&"tests/ps.db")?;

    Ok(())
}


