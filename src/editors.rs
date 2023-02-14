use crate::{CustomUi, Error};
use imgui::*;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use toml::*;
use uuid::Uuid;

pub trait ListItem {
	fn editor(&mut self, ui: &Ui);
	fn close(&mut self);
	fn is_new(&self) -> bool;
	fn uuid(&self) -> Uuid;
	fn name(&self) -> &String;
}

pub trait EditorList {
	type Item: ListItem + Hash;

	fn entries(&self) -> &Vec<Self::Item>;
	fn entries_mut(&mut self) -> &mut Vec<Self::Item>;
	fn add_entry(&mut self);
	fn unsaved(&mut self) -> &mut bool;
	fn search(&self) -> &str;
	fn search_mut(&mut self) -> &mut String;
	fn is_shown(&mut self) -> &mut bool;
}

#[derive(Hash)]
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
	fn unsaved(&mut self) -> &mut bool { &mut self.unsaved }
	fn search(&self) -> &str { &self.search_field }
	fn search_mut(&mut self) -> &mut String { &mut self.search_field }
	fn is_shown(&mut self) -> &mut bool { &mut self.is_shown }
}

#[derive(Hash)]
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
	fn unsaved(&mut self) -> &mut bool { &mut self.unsaved }
	fn search(&self) -> &str { &self.search_field }
	fn search_mut(&mut self) -> &mut String { &mut self.search_field }
	fn is_shown(&mut self) -> &mut bool { &mut self.is_shown }
}

#[derive(Hash)]
pub struct ClassData {
	pub texture_id: TextureId,
	// Data
	pub name: String,
	pub desc: String,
	// UI info
	pub uuid: Uuid,
	pub is_open: bool,
}

impl ClassData {
	pub fn with_texture(texture_id: TextureId) -> Self {
		Self {
			texture_id,
			name: String::new(),
			desc: String::new(),
			uuid: Uuid::new_v4(),
			is_open: true,
		}
	}

	pub fn to_toml(&self) -> Result<String, Error> {
		if self.name.len() == 0 {
			return Err(Error::from(
				"A class has a blank name."
			));
		}

		let mut toml = format!("[{:?}]\n", self.name);
		toml += &format!("desc = {:?}\n", self.desc);
		Ok(toml)
	}
}

impl ListItem for ClassData {
	fn editor(&mut self, ui: &Ui) {
		ui.input_text("##name", &mut self.name)
			.hint("Name")
			.build();

		ui.text("Description:");
		ui.input_text_multiline(
			"##desc",
			&mut self.desc,
			[ui.content_region_avail()[0], 64.0]
		).build();
	}

	fn close(&mut self) { self.is_open = false; }
	fn is_new(&self) -> bool {  self.name.len() == 0 }
	fn uuid(&self) -> Uuid { self.uuid }
	fn name(&self) -> &String { &self.name }
}

pub struct ClassEditor {
	pub unsaved: bool,
	pub is_shown: bool,
	pub classes: Vec<ClassData>,
	pub search_field: String,
	pub default_texture: TextureId,
}

impl ClassEditor {
	pub fn with_texture(default_texture: TextureId) -> Self {
		Self {
			unsaved: false,
			is_shown: true,
			classes: Vec::new(),
			search_field: String::new(),
			default_texture,
		}
	}

	pub fn open(path: impl AsRef<Path>, default_texture: TextureId) -> Self {
		let mut classes = Vec::new();

		if let Ok(toml) = fs::read_to_string(path) {
			let class_table: Table = toml.parse().unwrap();
			for (name, table) in class_table {
				let mut class = ClassData::with_texture(default_texture);
				class.name = name;
				if let Value::String(desc) = &table["desc"] {
					class.desc = desc.to_string()
				}
				classes.push(class);
			}
		}

		Self {
			unsaved: false,
			is_shown: true,
			classes,
			search_field: String::new(),
			default_texture,
		}
	}

	pub fn to_toml(&self) -> Result<String, Error> {
		let mut toml = String::new();
		for i in &self.classes {
			toml += &i.to_toml()?;
			toml += "\n";
		}
		Ok(toml)
	}
}

impl EditorList for ClassEditor {
	type Item = ClassData;
	
	fn entries(&self) -> &Vec<Self::Item> { &self.classes }
	fn entries_mut(&mut self) -> &mut Vec<Self::Item> { &mut self.classes }
	fn add_entry(&mut self) {
		self.classes.push(ClassData::with_texture(self.default_texture));
	}
	fn unsaved(&mut self) -> &mut bool { &mut self.unsaved }
	fn search(&self) -> &str { &self.search_field }
	fn search_mut(&mut self) -> &mut String { &mut self.search_field }
	fn is_shown(&mut self) -> &mut bool { &mut self.is_shown }
}