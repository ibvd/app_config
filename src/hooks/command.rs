use crate::hooks::Hook;
use serde_derive::Deserialize;
use std::io::Write;
use eyre::Result;
// use crate::config;

// CommandConf will store the user's input from the configuration file
// and then let us instantiate a File Object
#[derive(Debug, Deserialize)]
#[serde(rename = "command")]
pub struct CommandConf {
    pub command: String,
    pub pipe_data: Option<bool>,
}

impl CommandConf {
    pub fn convert(&self) -> Command {
        let p = match self.pipe_data {
            None => false,
            Some(x) => x,
        };
        Command::new(&self.command, p)
    }
}

/// The Command Hook will fire off an external script whenever new data is received
/// by the provider. Optionally, if pipe_data is true, it will pipe the data
/// received from the provider into the stdin pipe on the script.
#[derive(Debug, PartialEq)]
pub struct Command {
    command: String,
    pipe_data: bool,
}

impl Command {
    /// Create a new Command struct
    pub fn new(cmd: &str, pipe_data: bool) -> Command {
        Command {
            command: cmd.to_string(),
            pipe_data,
        }
    }
}

impl Hook for Command {
    /// Execute the command
    fn run(&self, data: &str) -> Result<()> {
        match self.pipe_data {
            // No data to pipe in.  Just run the command
            false => {
                let out = std::process::Command::new("/bin/bash")
                    .arg("-c")
                    .arg(self.command.clone())
                    .output()?;
                if !out.status.success() {
                    eprintln!("Failed to execute cmd: {}", self.command);
                    std::process::exit(exitcode::SOFTWARE);
                }
            }
            true => {
                // We have data to pipe in.  Spawn a process, send it data
                // Then check the return code
                let mut child = std::process::Command::new("/bin/bash")
                    .arg("-c")
                    .arg(self.command.clone())
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .expect("Failed to spawn child process");

                let stdin = child.stdin.as_mut().expect("Failed to open stdin");
                stdin.write_all(data.as_bytes())?;

                let output = child.wait_with_output()?;

                if !output.status.success() {
                    eprintln!("Failed to execute cmd: {}", self.command);
                    std::process::exit(exitcode::SOFTWARE);
                }
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd() {
        let c = Command::new(&"echo Booyeah", false);

        assert_eq!(c.run(&"").unwrap(), ());
    }

    #[test]
    fn test_piped_cmd() {
        let c = Command::new(&"echo", true);

        let res = c.run(&"Booyeah").unwrap();
        let expected = ();

        assert_eq!(res, expected);
    }

    fn gen_config() -> String {
        r#"
        [hooks.command]
         command = "cat > booyeah.txt"
         pipe_data = true
        "#
        .to_string()
    }

    #[test]
    fn parse_config() {
        let exp = Command::new(&"cat > booyeah.txt", true);

        let maps: toml::Value = toml::from_str(&gen_config()).unwrap();
        let conf: CommandConf = maps["hooks"]["command"].clone().try_into().unwrap();
        let res = conf.convert();

        assert_eq!(res, exp);
    }
}
