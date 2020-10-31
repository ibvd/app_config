use rusoto_appconfig::{AppConfig, GetConfigurationRequest};
use rusoto_core::Region;
use serde_derive::Deserialize;

use crate::providers::{BoxResult, Provider};

use rusqlite::{params, Connection};

/// AWSConf is used to parse a config file via serde and instantiate the
/// AWS Provider struct
#[derive(Debug, Deserialize)]
#[serde(rename = "AWS")]
pub struct AWSConf {
    pub application: String,
    pub environment: String,
    pub configuration: String,
    pub client_id: String,
    pub state_file: Option<String>,
}

impl AWSConf {
    pub fn convert(&self) -> AWS {
        AWS::new(
            &self.application,
            &self.environment,
            &self.configuration,
            &self.client_id,
            &self.state_file,
        )
    }
}

/// Provider for AWS AppConfig.  This allows us to check app config for updates
/// and cache any results into a local sqlite db.  The caching helps avoid charges
/// for polls when there are no new updates.
#[derive(Debug)]
pub struct AWS {
    application: String,
    environment: String,
    configuration: String,
    client_id: String,
    current_version: usize,
    db_conn: Connection,
}

impl AWS {
    /// Creates new AWS client
    /// The client will use the default user or system AWS credentials
    pub fn new(
        application: &str,
        environment: &str,
        configuration: &str,
        client_id: &str,
        state_file: &Option<String>,
    ) -> AWS {
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
        match AWS::create_cache(&conn) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Error, unable to create cache: {:?}", e);
                std::process::exit(exitcode::SOFTWARE);
            }
        };

        let version = match AWS::pull_latest_version(&conn) {
            Ok(ver) => ver as usize,
            Err(e) => {
                eprintln!("Error, unable to query cache: {:?}", e);
                std::process::exit(exitcode::SOFTWARE);
            }
        };

        // Create and return the Struct
        AWS {
            current_version: version,
            application: application.to_string(),
            environment: environment.to_string(),
            configuration: configuration.to_string(),
            client_id: client_id.to_string(),
            db_conn: conn,
        }
    }

    /// To avoid high charges the AWS AppConfig service needs us to supply
    /// the latest version of the config we have in cache.  
    /// This setup a sqlite table to store the version & data between runs
    fn create_cache(db_conn: &Connection) -> rusqlite::Result<()> {
        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS appConfig (
                id      INTEGER PRIMARY KEY,
                version INTEGER NOT NULL,
                data    varchar(15) NOT NULL
                )",
            params![],
        )?;
        db_conn.execute(
            "INSERT INTO appConfig (id, version, data) 
                SELECT 0, ?1, ?2
                WHERE NOT EXISTS (
                    SELECT * FROM appConfig WHERE id=0 )",
            params![0, ""],
        )?;
        Ok(())
    }

    /// Hit the local cache and pull out the latest version we have successfully
    /// loaded from the aws appConfig service
    fn pull_latest_version(db_conn: &Connection) -> rusqlite::Result<isize> {
        let res: isize = db_conn.query_row(
            "SELECT version FROM appConfig WHERE id=0",
            params![],
            |row| row.get(0),
        )?;
        Ok(res)
    }

    /// Store the latest data in the local cache
    fn update_cache(&self, version: usize, data: &str) -> rusqlite::Result<()> {
        let _stmt = self.db_conn.execute(
            "UPDATE appConfig SET
                            version = ?1, data = ?2
                            WHERE id=0",
            params![version as isize, data],
        )?;

        Ok(())
    }
}

impl Provider for AWS {
    /// Polls the AWS AppConfig service and checks for new data
    /// If we are up to date and already have the latest data
    /// returns None, else, retuns the new data
    /// Panics if we can not reach AWS, or check in with the service
    fn poll(&self) -> Option<String> {
        let request = GetConfigurationRequest {
            application: self.application.clone(),
            environment: self.environment.clone(),
            configuration: self.configuration.clone(),
            client_id: self.client_id.clone(),
            client_configuration_version: Some(self.current_version.to_string()),
        };

        let configuration = get_config(request);

        // Check if there was a new version, if not, do nothing
        let version = match configuration.configuration_version {
            None => {
                eprintln!("An error occurred - no data received.");
                std::process::exit(exitcode::UNAVAILABLE);
            }
            Some(version) => usize::from_str_radix(&version, 10).unwrap(),
        };

        if self.current_version == version {
            // We are up to date.  Nothing more to do
            return None;
        }

        // We have a new update.  Extract the data,
        // update local cache, and return the new data
        let data = std::str::from_utf8(&configuration.content.unwrap())
            .unwrap()
            .to_string();

        match self.update_cache(version, &data) {
            Ok(()) => {}
            Err(e) => eprintln!("Error saving to local cache: {:#?}", e),
        }

        Some(data)
    }

    /// Query
    /// Returns the latest version of the config from our local cache
    /// Does not contact the upstream source.
    fn query(&self) -> BoxResult<String> {
        let res: String =
            self.db_conn
                .query_row("SELECT data FROM appConfig WHERE id=0", params![], |row| {
                    row.get(0)
                })?;
        Ok(res)
    }
}

/// get_config()
/// Make the call to AWS appConfig and wait for the reply
#[tokio::main]
async fn get_config(request: GetConfigurationRequest) -> rusoto_appconfig::Configuration {
    let client = rusoto_appconfig::AppConfigClient::new(Region::default());

    let result = client.get_configuration(request).await;

    match result {
        // Ok(configuration) => configuration.unwrap(),
        Ok(configuration) => configuration,
        Err(e) => {
            eprintln!(
                "An error occurred - {:?} - when trying to fetch configuration",
                e
            );
            std::process::exit(exitcode::UNAVAILABLE);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn gen_aws_struct() -> AWS {
        AWS::new(&"myApp", &"dev", &"myConf", &"42", &None)
    }

    #[test]
    fn test_create_db() {
        let aws = gen_aws_struct();

        let res = AWS::create_cache(&aws.db_conn);
        assert_eq!(res, Ok(()));
    }

    #[test]
    fn test_pull_latest_version() {
        let aws = gen_aws_struct();

        let res = AWS::pull_latest_version(&aws.db_conn);
        assert_eq!(res, Ok(0));
    }

    #[test]
    fn test_update_cache() {
        let aws = gen_aws_struct();

        let res = AWS::pull_latest_version(&aws.db_conn);
        assert_eq!(res, Ok(0));

        let res = aws.update_cache(12, &"something");
        assert_eq!(res, Ok(()));

        let res = AWS::pull_latest_version(&aws.db_conn);
        assert_eq!(res, Ok(12));

        let res = aws.query().unwrap();
        assert_eq!(res, "something".to_string());
    }

    fn gen_config() -> String {
        r#"
        [providers.aws]
        application = "myApp"
        environment = "dev"
        configuration = "myConf"
        client_id = "42"
        "#
        .to_string()
    }

    #[test]
    fn parse_config() {
        let exp = AWS::new(&"myApp", &"dev", &"myConf", &"42", &None);
        let expected = format!("{:?}", exp);

        let maps: toml::Value = toml::from_str(&gen_config()).unwrap();
        let conf: AWSConf = maps["providers"]["aws"].clone().try_into().unwrap();
        let res = conf.convert();
        let result = format!("{:?}", res);

        assert_eq!(result, expected);
    }
}
