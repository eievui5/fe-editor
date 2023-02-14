mod custom_ui;
mod editors;
mod map;
mod texture_loader;

pub mod support;

use std::fmt::{self, Display, Formatter};

pub use custom_ui::*;
pub use editors::*;
pub use map::*;
pub use texture_loader::*;

pub use glium::backend::Facade;

#[derive(Debug)]
pub struct Error {
	pub msg: String,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		write!(f, "{}", self.msg)
	}
}

impl From<&str> for Error {
	fn from(s: &str) -> Self {
		Self {
			msg: s.to_string(),
		}
	}
}

impl std::error::Error for Error {}
