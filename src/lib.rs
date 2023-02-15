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

impl From<String> for Error {
	fn from(s: String) -> Self {
		Self {
			msg: s,
		}
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

pub struct SaveState {
	saved: bool,
	autosaved: bool,
}

impl SaveState {
	pub fn new() -> Self {
		Self {
			saved: true,
			autosaved: false,
		}
	}

	pub fn mark_unsaved(&mut self) {
		self.saved = false;
		self.autosaved = false;
	}

	pub fn mark_autosaved(&mut self) {
		self.autosaved = true;
	}

	pub fn mark_saved(&mut self) {
		self.saved = true;
	}

	pub fn is_saved(&self) -> bool {
		self.saved
	}

	pub fn is_autosaved(&self) -> bool {
		self.autosaved || self.saved
	}
}
