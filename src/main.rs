use imgui::*;
use uuid::Uuid;

mod support;

const WINDOW_NAME: &str = "Furry Emblem - Editor";

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

	pub fn is_new(&self) -> bool {
		self.name.len() == 0
	}
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

	pub fn is_new(&self) -> bool {
		self.name.len() == 0
	}
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

fn main() {
	let system = support::init(WINDOW_NAME);

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

		ui.window("Items")
			.position([32.0, 32.0], Condition::FirstUseEver)
			.size([200.0, 400.0], Condition::FirstUseEver)
			.menu_bar(true)
			.unsaved_document(item_editor.unsaved)
			.build(|| {
				ui.menu_bar(|| {
					ui.menu_item("Save");
				});

				ui.text("Search:");
				ui.input_text("##search", &mut item_editor.search_field).build();

				ui.separator();

				for item in &mut item_editor.items {
					if item_editor.search_field.len() > 0
						&& !item.name.to_ascii_lowercase().contains(&item_editor.search_field.to_ascii_lowercase())
					{
						continue;
					}

					let _id = ui.push_id(&item.uuid.to_string());

					ui.tree_node_config("##header")
						.label::<&str, &str>(&String::from(
							if item.name.len() > 0 {
								&item.name
							} else {
								"New Item"
							}))
						.framed(true)
						// Open the item entry if the name is empty,
						// since this means it's newly created; empty items can't be loaded from disk.
						.opened(item.is_new(), Condition::FirstUseEver)
						.build(|| {
							ui.input_text("##name", &mut item.name)
								.hint("Name")
								.build();

							ui.input_text("##desc", &mut item.desc)
								.hint("Description")
								.build();
						});


					//ui.separator();
				}

				if ui.button("Create New Item") {
					item_editor.items.push(ItemData::new());
				}
			});

		ui.window("Units")
			.position([32.0 * 2.0 + 200.0, 32.0], Condition::FirstUseEver)
			.size([200.0, 400.0], Condition::FirstUseEver)
			.menu_bar(true)
			.unsaved_document(unit_editor.unsaved)
			.build(|| {
				ui.menu_bar(|| {
					ui.menu_item("Save");
				});

				ui.text("Search:");
				ui.input_text("##search", &mut unit_editor.search_field).build();

				ui.separator();

				for unit in &mut unit_editor.units {
					if unit_editor.search_field.len() > 0
						&& !unit.name.to_ascii_lowercase().contains(&unit_editor.search_field.to_ascii_lowercase())
					{
						continue;
					}

					let _id = ui.push_id(&unit.uuid.to_string());

					ui.tree_node_config("##header")
						.label::<&str, &str>(&String::from(
							if unit.name.len() > 0 {
								&unit.name
							} else {
								"New Unit"
							}))
						.framed(true)
						// Open the item entry if the name is empty,
						// since this means it's newly created; empty items can't be loaded from disk.
						.opened(unit.is_new(), Condition::FirstUseEver)
						.build(|| {
							ui.input_text("##name", &mut unit.name)
								.hint("Name")
								.build();

							ui.input_text("##desc", &mut unit.desc)
								.hint("Description")
								.build();
						});
				}

				if ui.button("Create New Unit") {
					unit_editor.units.push(UnitData::new());
				}
			});
	});
}
