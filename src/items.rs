use crate::{CustomUi, ListItem, EditorList};
use imgui::*;
use uuid::Uuid;

pub struct ItemData {
	pub uuid: Uuid,
	/// Display name
	pub name: String,
	pub desc: String,
	pub unsaved_changes: bool,
	pub is_open: bool,
}

impl ItemData {
	pub fn new() -> Self {
		Self {
			uuid: Uuid::new_v4(),
			name: String::new(),
			desc: String::new(),
			unsaved_changes: true,
			is_open: true,
		}
	}
}

impl ListItem for ItemData {
	fn editor(&mut self, ui: &Ui) {
		ui.input_text("##name", &mut self.name)
			.hint("Name")
			.build();

		ui.input_text("##desc", &mut self.desc)
			.hint("Description")
			.build();
		ui.hover_tooltip("May be blank");
	}
	
	fn close(&mut self) { self.is_open = false; }
	fn is_new(&self) -> bool {  self.name.len() == 0 }
	fn uuid(&self) -> Uuid { self.uuid }
	fn name(&self) -> &String { &self.name }
}

pub struct ItemEditor {
	pub unsaved: bool,
	pub is_shown: bool,
	pub items: Vec<ItemData>,
	pub search_field: String,
}

impl ItemEditor {
	pub fn new() -> Self {
		Self {
			unsaved: false,
			is_shown: true,
			items: Vec::new(),
			search_field: String::new(),
		}
	}
}

impl EditorList for ItemEditor {
	type Item = ItemData;

	fn entries(&self) -> &Vec<Self::Item> { &self.items }
	fn entries_mut(&mut self) -> &mut Vec<Self::Item> { &mut self.items }
	fn add_entry(&mut self) { self.items.push(ItemData::new()); }
	fn has_changes(&self) -> bool { self.unsaved }
	fn search(&self) -> &str { &self.search_field }
	fn search_mut(&mut self) -> &mut String { &mut self.search_field }
	fn is_shown(&mut self) -> &mut bool { &mut self.is_shown }
}
