use crate::providers::Provider;
use serde_derive::Deserialize;
use eyre::{eyre, Result};
use rusqlite::{params, Connection};

use rusoto_ssm::{Ssm, SsmClient, GetParametersRequest};
use rusoto_core::Region;


// // // // // // // // // Handle Configuraion // // // // // // // //
#[derive(Debug, Deserialize)]
#[serde(rename = "param_store")]
pub struct ParamStoreConf {
    pub key: String,
    pub state_file: Option<String>,
}

impl ParamStoreConf {
    pub fn convert(&self) -> ParamStore {
        ParamStore::new(&self.key, &self.state_file)
    }
}


// // // // // // // // // // Provider // // // // // // // // // //

/// ParamStore povider polls an AWS SSM Parameter and triggers hooks
/// When the value changes from a previously cached value
#[derive(Debug)]
pub struct ParamStore {
    key: String,
    db_conn: Connection,
}

impl ParamStore {
    /// Creates new ParamStore provider
    pub fn new(key: &str, state_file: &Option<String>) -> ParamStore {

        // Open sqlitedb using in-memory if no file specified
        let conn = match state_file {
            &None => match Connection::open_in_memory() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error, unable to open in-memory db: {:?}", e);
                    std::process::exit(exitcode::SOFTWARE);
                }
            },
            Some(file_name) => match Connection::open(file_name) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error, unable to open state file {}: {:?}", file_name, e);
                    std::process::exit(exitcode::OSFILE);
                }
            },
        };

        // Setup the tables if they do not already exist
        match ParamStore::create_cache(&conn) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Error, unable to create cache: {:?}", e);
                std::process::exit(exitcode::SOFTWARE);
            }
        };

        ParamStore {
            key: key.to_string(),
            db_conn: conn,
        }
    }

    /// To know when the value of the parameter has changed, we need to 
    /// store the value locally. We will do so in a sqlite db.
    fn create_cache(db_conn: &Connection) -> rusqlite::Result<()> {
        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS param_store (
                id      INTEGER PRIMARY KEY,
                data    TEXT NOT NULL
                )",
            params![],
        )?;
        db_conn.execute(
            "INSERT INTO param_store (id, data) 
                SELECT 0, ?1
                WHERE NOT EXISTS (
                    SELECT * FROM param_store WHERE id=0 )",
            params![""],
        )?;
        Ok(())
    }

    /// Hit the local cache and pull out the latest data
    fn pull_latest_data(db_conn: &Connection) -> rusqlite::Result<String> {
        let res: String = db_conn.query_row(
            "SELECT data FROM param_store WHERE id=0",
            params![],
            |row| row.get(0),
        )?;
        Ok(res)
    }

    /// Store the latest data in the local cache
    fn update_cache(db_conn: &Connection, data: &str) -> rusqlite::Result<()> {
        let _stmt = db_conn.execute(
            "UPDATE param_store SET
                            data = ?1
                            WHERE id=0",
            params![data,],
        )?;

        Ok(())
    }
}

impl Provider for ParamStore {
    /// Just return the data contained in the Mock struct
    fn poll(&self) -> Result<Option<String>> {

        let value = get_params(&self.key)?;

        // Check for new data
        let old_value = ParamStore::pull_latest_data(&self.db_conn)?;
        if value == old_value {
            return Ok(None)
        }

        // We have new data, update the cache and return it
        ParamStore::update_cache(&self.db_conn, &value)?;
    
        Ok(Some(value))
    }

    /// Just return the data contained in the Mock struct
    fn query(&self) -> Result<String> {
        let res = ParamStore::pull_latest_data(&self.db_conn)?;
        Ok(res)
    }
}


/// get_params()
/// Make the call to SSM ParamStore and wait for the reply
#[tokio::main]
pub async fn get_params(key: &str) -> eyre::Result<String> {

    let request = GetParametersRequest {
        // names: vec![self.key.clone(),],
        names: vec![key.to_string(),],
        with_decryption: Some(true),
    };

    let client = SsmClient::new(Region::default());

    let result = match client.get_parameters(request).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error when fetching parameter: {:?}", e);
            std::process::exit(exitcode::UNAVAILABLE);
        }
    };

    let value: String = match result.parameters {
        None => return Err(eyre!("AWS Param Store returned no data")),
        Some(mut res) => match res.pop() {
            None => return Err(eyre!("AWS Param Store: parameter not found")),
            Some(param) => match param.value {
                None => return Err(eyre!("AWS Param Store value empty")),
                Some(value) => value,
            }
        }
    };

    Ok(value)
}


// // // // // // // // // // // Tests // // // // // // // // // // //
#[cfg(test)]
mod test {
    use super::*;

    fn gen_ps_struct() -> ParamStore {
        ParamStore::new(&"Hello", &None)
    }

    #[test]
    fn test_create_db() {
        let p = gen_ps_struct();

        let res = ParamStore::create_cache(&p.db_conn);
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn test_db_updates() {
        let p = gen_ps_struct();

        let res = ParamStore::create_cache(&p.db_conn);
        assert_eq!(res, Ok(()));

        let res = ParamStore::pull_latest_data(&p.db_conn);
        assert_eq!(res, Ok("".to_string()));

        let res = ParamStore::update_cache(&p.db_conn, &"Yo");
        assert_eq!(res, Ok(()));

        let res = ParamStore::pull_latest_data(&p.db_conn);
        assert_eq!(res, Ok("Yo".to_string()));
    }


    #[test]
    fn test_poll() {
        let p = gen_ps_struct();

        let res = p.query().unwrap();
        assert_eq!(res, String::from(""));
    }

    fn gen_config() -> String {
        r#"
        [providers.param_store]
        key = "Hello"
        "#
        .to_string()
    }

    #[test]
    fn parse_config() {
        let exp = ParamStore::new(&"Hello", &None);
        let expected = format!("{:?}", exp);

        let maps: toml::Value = toml::from_str(&gen_config()).unwrap();
        let conf: ParamStoreConf = maps["providers"]["param_store"]
                                    .clone().try_into().unwrap();
        let res = conf.convert();
        let result = format!("{:?}", res);

        assert_eq!(result, expected);
    }
}
