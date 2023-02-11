mod custom_ui;
mod items;
mod units;

pub mod support;

use imgui::*;
use uuid::Uuid;

pub use items::*;
pub use units::*;
pub use custom_ui::*;

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
