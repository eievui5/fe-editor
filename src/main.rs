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

fn main() {
	let system = support::init(WINDOW_NAME);

	let mut item_editor = ItemEditor::new();

	system.main_loop(move |_, ui| {
		let display_size = ui.io().display_size;

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
			.position([display_size[0] - display_size[0] / 5.0 - 16.0, 32.0], Condition::Always)
			.size([display_size[0] / 5.0, display_size[1] - 48.0], Condition::Always)
			.no_decoration()
			.title_bar(true)
			.movable(false)
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

					ui.input_text("##name", &mut item.name)
						.hint("Name")
						.build();

					ui.input_text("##desc", &mut item.desc)
						.hint("Description")
						.build();

					ui.separator();
				}

				if ui.button("Create New Item") {
					item_editor.items.push(ItemData::new());
				}
			});
	});
}
