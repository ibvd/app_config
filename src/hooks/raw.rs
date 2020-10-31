use crate::hooks::Hook;
use serde_derive::Deserialize;
use eyre::Result;

// RawConf will let the config file parser instantiate a Raw Hook struct
// Overkill for this simpel module, but some other hooks are more complex and
// require the second level of abstraction. It is easier to make them all consistent
#[derive(Debug, Deserialize)]
#[serde(rename = "raw")]
pub struct RawConf {}

impl RawConf {
    pub fn convert(&self) -> Raw {
        Raw {}
    }
}

#[derive(Debug, Deserialize, PartialEq)]
/// Raw allows us to output the data received from the provider directly
/// to stdout
pub struct Raw {}

impl Hook for Raw {
    /// Write the raw data to stdout
    fn run(&self, data: &str) -> Result<()> {
        println!("{}", data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_config() -> String {
        "[hooks.raw]".to_string()
    }

    #[test]
    fn parse_config() {
        let exp = Raw {};

        let maps: toml::Value = toml::from_str(&gen_config()).unwrap();
        let conf: RawConf = maps["hooks"]["raw"].clone().try_into().unwrap();
        let res: Raw = conf.convert();

        assert_eq!(res, exp);
    }
}
