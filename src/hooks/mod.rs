pub mod template;
pub use crate::hooks::template::{Template, TemplateConf};
pub mod file;
pub use crate::hooks::file::{File, FileConf};
pub mod raw;
pub use crate::hooks::raw::{Raw, RawConf};
pub mod command;
pub use crate::hooks::command::{Command, CommandConf};

/*
use std::error::Error;
type BoxResult<T> = Result<T, Box<dyn Error>>;
*/
use eyre::Result;

pub trait Hook: std::fmt::Debug {
    fn run(&self, data: &str) -> Result<()>;
    // fn run(&self, data: &str) -> BoxResult<()>;
}
