use crate::*;
use fe_data::FeError;
use imgui::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::hash::Hasher;
use std::path::{Path, PathBuf};
use toml::*;
use uuid::Uuid;

use std::hash::Hash;
pub type ClassIcons = HashMap<PathBuf, TextureId>;

#[derive(Hash)]
pub struct ClassEntry {
	// Data
	pub data: ClassData,
	pub uuid: Uuid,
	pub is_open: bool,
}

impl ClassEntry {
	pub fn from(name: String, table: Table) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			data: ClassData::from(name, table)?,
			uuid: Uuid::new_v4(),
			is_open: true,
		})
	}

	pub fn with_texture(texture: PathBuf) -> Self {
		Self {
			data: ClassData::with_texture(texture),
			uuid: Uuid::new_v4(),
			is_open: true,
		}
	}

	fn editor(&mut self, ui: &Ui, class_icons: &ClassIcons) {
		ui.input_text("##name", &mut self.data.name).hint("Name").build();
		if ui.image_button("##class", class_icons[&self.data.texture], [32.0, 32.0]) {
			ui.open_popup("Select Icon");
		}
		ui.hover_tooltip("Click to select class icon");

		ui.text("Description:");
		ui.input_text_multiline(
			"##desc",
			&mut self.data.desc,
			[ui.content_region_avail()[0], 64.0],
		)
		.build();

		ui.popup("Select Icon", || {
			ui.text("Select an icon");
			for (i, (path, texture)) in class_icons.iter().enumerate() {
				// Classes per row.
				if i % 3 != 0 {
					ui.same_line();
				}
				if ui.image_button(
					i.to_string(),
					*texture,
					[32.0; 2]
				) {
					self.data.texture = path.clone();
				}
				ui.hover_tooltip(&path.to_string_lossy());
			}
		});
	}

	fn close(&mut self) {
		self.is_open = false;
	}

	fn is_new(&self) -> bool {
		self.data.name.len() == 0
	}
}

pub struct ClassEditor {
	pub unsaved: bool,
	pub classes: Vec<ClassEntry>,
	pub search_field: String,
	pub default_icon: PathBuf,
}

impl ClassEditor {
	pub fn open(path: impl AsRef<Path>, default_icon: PathBuf) -> Result<Self, Box<dyn Error>> {
		let mut classes = Vec::new();

		if let Ok(toml) = fs::read_to_string(path) {
			let class_table: Table = toml.parse()?;
			for (name, table) in class_table {
				let class = if let Value::Table(table) = table {
					ClassEntry::from(name, table)?
				} else {
					Err(FeError::from("Class data is not a table"))?
				};
				classes.push(class);
			}
		}

		Ok(Self {
			unsaved: false,
			classes,
			search_field: String::new(),
			default_icon,
		})
	}

	pub fn to_toml(&self) -> Result<String, Box<dyn Error>> {
		let mut toml = String::new();
		for i in &self.classes {
			toml += &i.data.to_toml()?;
			toml += "\n";
		}
		Ok(toml)
	}

	fn add_entry(&mut self) {
		self.classes
			.push(ClassEntry::with_texture(self.default_icon.clone()));
	}

	pub fn draw(&mut self, ui: &Ui, position: (f32, f32), class_icons: &ClassIcons) {
		// Track any changes that occur during this frame.
		let mut editor_hash = DefaultHasher::new();
		self.classes.hash(&mut editor_hash);
		let editor_hash = editor_hash.finish();

		ui.window("Classes")
			.position([position.0, position.1], Condition::FirstUseEver)
			.size([200.0, 400.0], Condition::FirstUseEver)
			.menu_bar(true)
			.focus_on_appearing(false)
			.collapsed(true, Condition::FirstUseEver)
			.unsaved_document(self.unsaved)
			.build(|| {
				ui.menu_bar(|| {
					ui.menu_item("Save");
				});

				ui.text("Search:");
				ui.input_text("##search", &mut self.search_field).build();

				ui.separator();

				let normalized_query = if self.search_field.len() > 0 {
					Some(self.search_field.to_ascii_lowercase())
				} else {
					None
				};

				for item in &mut self.classes {
					if let Some(query) = &normalized_query {
						if !item.data.name.to_ascii_lowercase().contains(query) {
							continue;
						}
					}

					let _id = ui.push_id(&item.uuid.to_string());

					ui.tree_node_config("##header")
						.label::<String, String>(if !item.is_new() {
							item.data.name.clone()
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
									item.data.name
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
		self.classes.hash(&mut current_hash);
		let current_hash = current_hash.finish();

		if !self.unsaved {
			self.unsaved = editor_hash != current_hash;
		}
	}
}
