use crate::providers::Provider;
use serde_derive::Deserialize;
use eyre::Result;

#[derive(Debug, Deserialize)]
#[serde(rename = "mock")]
pub struct MockConf {
    pub data: String,
}

impl MockConf {
    pub fn convert(&self) -> Mock {
        Mock::new(&self.data)
    }
}

/// Mock is a dummy provider that just returns whatever data it was given
/// It is mainly useful for dialing in templates as it lets you quickly
/// test input data against the desired output format
#[derive(Debug, PartialEq)]
pub struct Mock {
    data: String,
}

impl Mock {
    /// Creates new Mock provider
    pub fn new(data: &str) -> Mock {
        Mock {
            data: data.to_string(),
        }
    }
}

impl Provider for Mock {
    /// Just return the data contained in the Mock struct
    fn poll(&self) -> Result<Option<String>> {
        Ok(Some(self.data.clone()))
    }

    /// Just return the data contained in the Mock struct
    fn query(&self) -> Result<String> {
        Ok(self.data.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn gen_mock_struct() -> Mock {
        Mock::new(&"Am I a mock")
    }

    #[test]
    fn test_poll() {
        let mock = gen_mock_struct();

        let res = mock.poll().unwrap().unwrap();
        assert_eq!(res, String::from("Am I a mock"));

        let res = mock.query().unwrap();
        assert_eq!(res, String::from("Am I a mock"));
    }

    fn gen_config() -> String {
        r#"
        [providers.mock]
        data = "Am I a mock"
        "#
        .to_string()
    }

    #[test]
    fn parse_config() {
        let exp = Mock::new(&"Am I a mock");

        let maps: toml::Value = toml::from_str(&gen_config()).unwrap();
        let conf: MockConf = maps["providers"]["mock"].clone().try_into().unwrap();
        let res = conf.convert();

        assert_eq!(res, exp);
    }
}
