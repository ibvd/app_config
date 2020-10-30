// reexport AWS as part of this module
pub mod aws;
pub use crate::providers::aws::{AWS, AWSConf};
pub mod mock;
pub use crate::providers::mock::{Mock, MockConf};

use std::error::Error;
type BoxResult<T> = Result<T, Box<dyn Error>>;

pub trait Provider: std::fmt::Debug {
    fn poll(&self)  -> Option<String>;

    // fn query(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn query(&self) -> BoxResult<String>;
}
