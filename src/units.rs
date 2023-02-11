use crate::{ListItem, EditorList};
use imgui::*;
use uuid::Uuid;

pub struct UnitData {
	pub uuid: Uuid,
	/// Display name
	pub name: String,
	pub desc: String,
	pub class: String,
	pub unsaved_changes: bool,
	pub is_open: bool,
}

impl UnitData {
	pub fn new() -> Self {
		Self {
			uuid: Uuid::new_v4(),
			name: String::new(),
			desc: String::new(),
			class: String::new(),
			unsaved_changes: true,
			is_open: true,
		}
	}
}

impl ListItem for UnitData {
	fn editor(&mut self, ui: &Ui) {
		ui.input_text("##name", &mut self.name)
			.hint("Name")
			.build();

		ui.text("Background:");
		ui.input_text_multiline(
			"##desc",
			&mut self.desc,
			[ui.content_region_avail()[0], 64.0]
		).build();

		ui.input_text("##class", &mut self.class)
			.hint("Class")
			.build();
	}

	fn close(&mut self) { self.is_open = false; }
	fn is_new(&self) -> bool {  self.name.len() == 0 }
	fn uuid(&self) -> Uuid { self.uuid }
	fn name(&self) -> &String { &self.name }
}

pub struct UnitEditor {
	pub unsaved: bool,
	pub is_shown: bool,
	pub units: Vec<UnitData>,
	pub search_field: String,
}

impl UnitEditor {
	pub fn new() -> Self {
		Self {
			unsaved: false,
			is_shown: true,
			units: Vec::new(),
			search_field: String::new(),
		}
	}
}

impl EditorList for UnitEditor {
	type Item = UnitData;
	
	fn entries(&self) -> &Vec<Self::Item> { &self.units }
	fn entries_mut(&mut self) -> &mut Vec<Self::Item> { &mut self.units }
	fn add_entry(&mut self) { self.units.push(UnitData::new()); }
	fn has_changes(&self) -> bool { self.unsaved }
	fn search(&self) -> &str { &self.search_field }
	fn search_mut(&mut self) -> &mut String { &mut self.search_field }
	fn is_shown(&mut self) -> &mut bool { &mut self.is_shown }
}
