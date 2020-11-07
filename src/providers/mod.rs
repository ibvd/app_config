pub mod appcfg;
pub use crate::providers::appcfg::{AppCfgConf, AppCfg};
pub mod mock;
pub use crate::providers::mock::{Mock, MockConf};
pub mod param_store;
pub use crate::providers::param_store::{ParamStore, ParamStoreConf};

use eyre::Result;

pub trait Provider: std::fmt::Debug {
    fn poll(&self) -> Result<Option<String>>;

    fn query(&self) -> Result<String>;
}
