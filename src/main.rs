use imgui::*;
use uuid::Uuid;

mod support;

trait ListItem {
	fn editor(&mut self, ui: &Ui);
	fn close(&mut self);
	fn is_new(&self) -> bool;
	fn uuid(&self) -> Uuid;
	fn name(&self) -> &String;
}

trait EditorList {
	type Item: ListItem;
	fn entries(&self) -> &Vec<Self::Item>;
	fn entries_mut(&mut self) -> &mut Vec<Self::Item>;
	fn add_entry(&mut self);
	fn has_changes(&self) -> bool;
	fn search(&self) -> &str;
	fn search_mut(&mut self) -> &mut String;
}

trait CustomUi {
	fn editor_list<T: EditorList>(&self, editor: &mut T, hint: &str, position: (f32, f32));
}

impl CustomUi for Ui {
	fn editor_list<T: EditorList>(&self, editor: &mut T, hint: &str, position: (f32, f32)) {
		self.window(hint)
			.position([position.0, position.1], Condition::FirstUseEver)
			.size([200.0, 400.0], Condition::FirstUseEver)
			.menu_bar(true)
			.unsaved_document(editor.has_changes())
			.build(|| {
				self.menu_bar(|| {
					self.menu_item("Save");
				});

				self.text("Search:");
				self.input_text("##search", &mut editor.search_mut()).build();

				self.separator();

				let normalized_query = if editor.search().len() > 0 {
					Some(editor.search().to_ascii_lowercase())
				} else {
					None
				};

				for item in editor.entries_mut() {
					if let Some(query) = &normalized_query {
						if !item.name().to_ascii_lowercase().contains(query) {
							continue;
						}
					}

					let _id = self.push_id(&item.uuid().to_string());

					self.tree_node_config("##header")
						.label::<String, String>(
							if item.name().len() > 0 {
								item.name().clone()
							} else {
								format!("New {hint}")
							}
						)
						.framed(true)
						// Open the item entry if the name is empty,
						// since this means it's newly created; empty items can't be loaded from disk.
						.opened(item.is_new(), Condition::FirstUseEver)
						.build(|| {
							item.editor(&self);

							if self.button("Delete") {
								self.open_popup("Delete");
							}

							if self.modal_popup_config("Delete").build(|| {
								self.text_wrapped(&format!(
									"Do you really want to delete \"{}\"?",
									item.name()
								));
								if self.button("Cancel") {
									self.close_current_popup();
								}
								self.same_line();
								if self.button("Delete") {
									self.close_current_popup();
									return true;
								}
								false
							}) == Some(true) {
								item.close()
							}
						});

					self.separator();
				}

				if self.button(&format!("Create New {hint}")) {
					editor.add_entry();
				}
			});
	}
}

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
	}
	
	fn close(&mut self) { self.is_open = false; }
	fn is_new(&self) -> bool {  self.name.len() == 0 }
	fn uuid(&self) -> Uuid { self.uuid }
	fn name(&self) -> &String { &self.name }
}

pub struct UnitData {
	pub uuid: Uuid,
	/// Display name
	pub name: String,
	pub desc: String,
	pub unsaved_changes: bool,
	pub is_open: bool,
}

impl UnitData {
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

impl ListItem for UnitData {
	fn editor(&mut self, ui: &Ui) {
		ui.input_text("##name", &mut self.name)
			.hint("Name")
			.build();

		ui.input_text("##desc", &mut self.desc)
			.hint("Description")
			.build();
	}

	fn close(&mut self) { self.is_open = false; }
	fn is_new(&self) -> bool {  self.name.len() == 0 }
	fn uuid(&self) -> Uuid { self.uuid }
	fn name(&self) -> &String { &self.name }
}

pub struct ItemEditor {
	pub unsaved: bool,
	pub items: Vec<ItemData>,
	pub search_field: String,
}

impl ItemEditor {
	pub fn new() -> Self {
		Self {
			unsaved: false,
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
}

pub struct UnitEditor {
	pub unsaved: bool,
	pub units: Vec<UnitData>,
	pub search_field: String,
}

impl UnitEditor {
	pub fn new() -> Self {
		Self {
			unsaved: false,
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
}

fn main() {
	let system = support::init("Furry Emblem - Editor");

	let mut item_editor = ItemEditor::new();
	let mut unit_editor = UnitEditor::new();

	system.main_loop(move |_, ui| {
		ui.main_menu_bar(|| {
			ui.menu("File", || {
				if ui.menu_item("Open") {
					println!("Opening file");
				}
				ui.separator();
				ui.menu_item("New...");
			});
			ui.menu_item("Options");
			ui.menu("Help", || {
				ui.text("Furry Emblem Editor");
				ui.text("By Evie M.");
			});
		});

		ui.editor_list(
			&mut item_editor,
			"Item",
			(32.0, 32.0),
		);

		ui.editor_list(
			&mut unit_editor,
			"Unit",
			(32.0 * 2.0 + 200.0, 32.0),
		);

		// End-of-frame cleanup
		item_editor.items.retain(|i| i.is_open);
		unit_editor.units.retain(|i| i.is_open);
	});
}
