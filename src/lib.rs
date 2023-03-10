mod classes;
mod custom_ui;
mod map;
mod texture_loader;

pub mod support;

pub use classes::*;
pub use custom_ui::*;
pub use fe_data::*;
pub use map::*;
pub use texture_loader::*;

pub use glium::backend::Facade;

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
