use crate::*;
use fe_data::FeError;
use imgui::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::Hasher;
use std::path::{Path, PathBuf};
use toml::*;
use uuid::Uuid;

use std::hash::Hash;
pub type ClassIcons = HashMap<PathBuf, TextureId>;

#[derive(Hash)]
pub struct ClassData {
	// Data
	pub name: String,
	pub desc: String,
	pub texture: PathBuf,
	pub uuid: Uuid,
	pub is_open: bool,
}

impl ClassData {
	// TODO: This really should not need a copy, but the main loop closure seems
	// to make it so that these collections can't be "safely" borrowed.
	pub fn with_texture(texture: PathBuf) -> Self {
		Self {
			texture,
			name: String::new(),
			desc: String::new(),
			uuid: Uuid::new_v4(),
			is_open: true,
		}
	}

	pub fn to_toml(&self) -> Result<String, FeError> {
		if self.name.len() == 0 {
			return Err(FeError::from("A class has a blank name."));
		}

		let mut toml = format!("[{:?}]\n", self.name);
		toml += &format!("desc = {:?}\n", self.desc);
		Ok(toml)
	}

	fn editor(&mut self, ui: &Ui, class_icons: &ClassIcons) {
		ui.input_text("##name", &mut self.name).hint("Name").build();
		if ui.image_button("##class", class_icons[&self.texture], [32.0, 32.0]) {
			ui.open_popup("Select Icon");
		}
		ui.hover_tooltip("Click to select class icon");

		ui.text("Description:");
		ui.input_text_multiline(
			"##desc",
			&mut self.desc,
			[ui.content_region_avail()[0], 64.0],
		)
		.build();

		//ui.popup("Select Icon", || {
		//	ui.text("Select an icon");
		//	for (i, texture) in self.textures.iter().enumerate() {
		//		// Classes per row.
		//		if i % 3 != 0 {
		//			ui.same_line();
		//		}
		//		if ui.image_button(
		//			i.to_string(),
		//			*texture,
		//			[32.0; 2]
		//		) {
		//			self.texture = *texture;
		//		}
		//	}
		//});
	}

	fn close(&mut self) {
		self.is_open = false;
	}
	fn is_new(&self) -> bool {
		self.name.len() == 0
	}
	fn uuid(&self) -> Uuid {
		self.uuid
	}
	fn name(&self) -> &String {
		&self.name
	}
}

pub struct ClassEditor {
	pub unsaved: bool,
	pub classes: Vec<ClassData>,
	pub search_field: String,
	pub default_icon: PathBuf,
}

impl ClassEditor {
	pub fn open(path: impl AsRef<Path>, default_icon: PathBuf) -> Self {
		let mut classes = Vec::new();

		if let Ok(toml) = fs::read_to_string(path) {
			let class_table: Table = toml.parse().unwrap();
			for (name, table) in class_table {
				// TODO: This texture should be loaded from classes.toml
				let mut class = ClassData::with_texture(default_icon.clone());
				class.name = name;
				if let Value::String(desc) = &table["desc"] {
					class.desc = desc.to_string()
				}
				classes.push(class);
			}
		}

		Self {
			unsaved: false,
			classes,
			search_field: String::new(),
			default_icon,
		}
	}

	pub fn to_toml(&self) -> Result<String, FeError> {
		let mut toml = String::new();
		for i in &self.classes {
			toml += &i.to_toml()?;
			toml += "\n";
		}
		Ok(toml)
	}

	fn entries(&self) -> &Vec<ClassData> {
		&self.classes
	}
	fn entries_mut(&mut self) -> &mut Vec<ClassData> {
		&mut self.classes
	}
	fn add_entry(&mut self) {
		self.classes
			.push(ClassData::with_texture(self.default_icon.clone()));
	}
	fn unsaved(&mut self) -> &mut bool {
		&mut self.unsaved
	}
	fn search(&self) -> &str {
		&self.search_field
	}
	fn search_mut(&mut self) -> &mut String {
		&mut self.search_field
	}

	pub fn draw(&mut self, ui: &Ui, position: (f32, f32), class_icons: &ClassIcons) {
		// Track any changes that occur during this frame.
		let mut editor_hash = DefaultHasher::new();
		self.entries().hash(&mut editor_hash);
		let editor_hash = editor_hash.finish();

		ui.window("Classes")
			.position([position.0, position.1], Condition::FirstUseEver)
			.size([200.0, 400.0], Condition::FirstUseEver)
			.menu_bar(true)
			.focus_on_appearing(false)
			.collapsed(true, Condition::FirstUseEver)
			.unsaved_document(*self.unsaved())
			.build(|| {
				ui.menu_bar(|| {
					ui.menu_item("Save");
				});

				ui.text("Search:");
				ui.input_text("##search", &mut self.search_mut()).build();

				ui.separator();

				let normalized_query = if self.search().len() > 0 {
					Some(self.search().to_ascii_lowercase())
				} else {
					None
				};

				for item in self.entries_mut() {
					if let Some(query) = &normalized_query {
						if !item.name().to_ascii_lowercase().contains(query) {
							continue;
						}
					}

					let _id = ui.push_id(&item.uuid().to_string());

					ui.tree_node_config("##header")
						.label::<String, String>(if item.name().len() > 0 {
							item.name().clone()
						} else {
							format!("New class")
						})
						.framed(true)
						// Open the item entry if the name is empty,
						// since this means it's newly created; empty items can't be loaded from disk.
						.opened(item.is_new(), Condition::FirstUseEver)
						.build(|| {
							item.editor(&ui, &class_icons);
							if ui.button("Delete") {
								ui.open_popup("Delete");
							}

							if ui.modal_popup_config("Delete").build(|| {
								ui.text(&format!(
									"Do you really want to delete \"{}\"?",
									item.name()
								));
								if ui.button("Cancel") {
									ui.close_current_popup();
								}
								ui.same_line();
								if ui.button("Delete") {
									ui.close_current_popup();
									return true;
								}
								false
							}) == Some(true)
							{
								item.close()
							}
						});

					ui.separator();
				}

				if ui.button(&format!("Create new class")) {
					self.add_entry();
				}
			});

		let mut current_hash = DefaultHasher::new();
		self.entries().hash(&mut current_hash);
		let current_hash = current_hash.finish();

		if !*self.unsaved() {
			*self.unsaved() = editor_hash != current_hash;
		}
	}
}
