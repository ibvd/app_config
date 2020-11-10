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


// // // // // // // // CLI // // // // // // // //

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("check").arg("-f").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}


// // // // // // Config File Parsing // // // // // // 

#[test]
fn invalid_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("check")
        .arg("-f")
        .arg("./tests/invalid_config.toml");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse"));

    Ok(())
}

#[test]
fn missing_field() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("check").arg("-f").arg("./tests/missing_field.toml");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse"));

    Ok(())
}


// // // // // // Mock Provider // // // // // // 

#[test]
fn test_mock_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("check").arg("-f").arg("./tests/mock.toml");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Where am I"));

    Ok(())
}

#[test]
fn test_mock_query() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("app_config")?;

    cmd.arg("query").arg("-f").arg("./tests/mock.toml");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Where am I"));

    Ok(())
}

// // // // // // Parameter Store // // // // // // 


// // // // // // // File Hook // // // // // // //

#[test]
fn test_file_hook() -> Result<(), Box<dyn std::error::Error>> {
    // This also tests our mock provider

    let outfile = "./tests/raw_output.txt";

    // Ensure raw_output.txt is removed prior to our test
    rm_file(&outfile)?;

    // Run app_config with file hook
    let mut cmd = Command::cargo_bin("app_config")?;
    cmd.arg("check").arg("-f").arg("./tests/file_hook.toml");
    cmd.assert().success();

    // Test output is as expected
    let cmd = Command::new("/bin/bash")
        .arg("-c")
        .arg("cat ./tests/raw_output.txt")
        .output()
        .expect("failed to cat ./tests/raw_output.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Where am I"));

    // Ensure raw_output.txt is removed post our test
    rm_file(&outfile)?;

    Ok(())
}


// // // // // // Template Hook // // // // // // 

#[test]
fn test_template_hook() -> Result<(), Box<dyn std::error::Error>> {
    let outfile = "./tests/rendered.txt";

    // Ensure rendered.txt is removed prior to our test
    rm_file(&outfile)?;

    // Run app_config with file hook
    let mut cmd = Command::cargo_bin("app_config")?;
    cmd.arg("check").arg("-f").arg("./tests/template_hook.toml");
    cmd.assert().success();

    // Test output is as expected
    let cmd = Command::new("/bin/bash")
        .arg("-c")
        .arg("cat ./tests/rendered.txt")
        .output()
        .expect("failed to cat ./tests/rendered.txt");
    cmd.assert().success().stdout(predicate::str::similar(
        "
[Peer]
EndPoint = host1
PublicKey = xyz

[Peer]
EndPoint = host2
PublicKey = abc

",
    ));

    // Ensure rendered.txt is removed post our test
    rm_file(&outfile)?;

    Ok(())
}

#[test]
fn test_template_stdout() -> Result<(), Box<dyn std::error::Error>> {
    // Run app_config with file hook
    let mut cmd = Command::cargo_bin("app_config")?;
    cmd.arg("check")
        .arg("-f")
        .arg("./tests/template_stdout.toml");
    cmd.assert().success().stdout(predicate::str::similar(
        "
[Peer]
EndPoint = host1
PublicKey = xyz

[Peer]
EndPoint = host2
PublicKey = abc

",
    ));

    Ok(())
}

#[test]
fn test_template_and_raw_stdout() -> Result<(), Box<dyn std::error::Error>> {
    // Run app_config with file hook
    let mut cmd = Command::cargo_bin("app_config")?;
    cmd.arg("check")
        .arg("-f")
        .arg("./tests/template_raw_stdout.toml");
    cmd.assert().success().stdout(predicate::str::similar(
        "
[Peer]
EndPoint = host1
PublicKey = xyz

[Peer]
EndPoint = host2
PublicKey = abc

---
hosts:
  - name: host1
    public_key: xyz
  - name: host2
    public_key: abc
",
    ));

    Ok(())
}


// // // // // // Command Hook // // // // // // 

#[test]
fn test_garbage_cmd() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("app_config")?;
    cmd.arg("check")
        .arg("-f")
        .arg("./tests/command_garbage.toml");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to execute cmd: /not/a/command",
    ));

    Ok(())
}

#[test]
fn test_piped_cmd() -> Result<(), Box<dyn std::error::Error>> {
    let outfile = &"./tests/piped.txt";

    // Ensure outfile is removed prior to our test
    rm_file(outfile)?;

    // Run the test
    let mut cmd = Command::cargo_bin("app_config")?;
    cmd.arg("check").arg("-f").arg("./tests/command_piped.toml");
    cmd.assert().success();

    // Test output is as expected
    let cmd = Command::new("/bin/bash")
        .arg("-c")
        .arg("cat ./tests/piped.txt")
        .output()
        .expect("failed to cat ./tests/piped.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Where am I"));

    // Ensure outfile is removed post our test
    rm_file(outfile)?;

    Ok(())
}
