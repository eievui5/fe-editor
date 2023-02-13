mod custom_ui;
mod items;
mod texture_loader;
mod units;

pub mod support;

use imgui::*;
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

pub use custom_ui::*;
pub use items::*;
pub use texture_loader::*;
pub use units::*;

pub use glium::backend::Facade;

pub trait ListItem {
	fn editor(&mut self, ui: &Ui);
	fn close(&mut self);
	fn is_new(&self) -> bool;
	fn uuid(&self) -> Uuid;
	fn name(&self) -> &String;
}

pub trait EditorList {
	type Item: ListItem;
	fn entries(&self) -> &Vec<Self::Item>;
	fn entries_mut(&mut self) -> &mut Vec<Self::Item>;
	fn add_entry(&mut self);
	fn has_changes(&self) -> bool;
	fn search(&self) -> &str;
	fn search_mut(&mut self) -> &mut String;
	fn is_shown(&mut self) -> &mut bool;
}

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
