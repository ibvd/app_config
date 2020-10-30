use serde_derive::Deserialize;
use crate::hooks::{Hook, BoxResult};
// use crate::config;

use std::fs;
use std::io::prelude::*;
use shellexpand::tilde;

// FileConf will store the user's input from the configuration file
// and then let us instantiate a File Object
// We do not need that here, but some other hooks are more complex and require
// the second level of abstraction, so it is easier to make them all consistent
#[derive(Debug, Deserialize)]
#[serde(rename="File")]
pub struct FileConf {
    pub outfile: String,
}

impl FileConf {
    pub fn convert(&self) -> File {
        File::new(&self.outfile)
    }
}

/// File
/// This hook allow us to take the raw data feed from a Provider and write it to
/// a text file stored in <outfile>
#[derive(Debug, PartialEq, Deserialize)]
pub struct File {
    outfile: String,
}

impl File {
    /// Create a new File struct
    pub fn new(outfile: &str) -> File {
        // Read in the template from the provided file.
        let expanded_path = String::from(tilde(outfile));

        File { 
            outfile: expanded_path,
        }
    }
}


impl Hook for File {
    /// Write the raw data to the output file
    fn run(&self, data: &str) -> BoxResult<()> {

        // If the user configured 'outfile', write the template there
        // Else print the rendered templete to stdout
        match fs::File::create(&self.outfile) {
            Ok(mut file_handle) => 
                file_handle.write_all(data.as_bytes())?,
            Err(e) => {
                eprintln!("Could not open {}: {}", self.outfile, e);
                std::process::exit(exitcode::OSFILE);
            },
        };
        Ok(())
    }
}


#[cfg(test)]
mod tests { 
    use super::*;

    fn gen_config() -> String {
        "[hooks.file]
         outfile = \"somefile.txt\"
        ".to_string()
    }

    #[test]
    fn parse_config() {
        let exp = File::new(&"somefile.txt");

        let maps: toml::Value = toml::from_str(&gen_config()).unwrap();
        let conf: FileConf = maps["hooks"]["file"].clone().try_into().unwrap();
        let res: File = conf.convert();

        assert_eq!(res, exp);
    }
}

