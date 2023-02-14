use std::error::Error;
use std::fs;
use std::path::Path;
use furry_emblem_editor::*;
use imgui::*;

const AUTOSAVE_FREQUENCY: f32 = 2.0;
const MAIN_MENU_HEIGHT: f32 = 22.0;
const MAP_VIEWER_MARGIN: f32 = 32.0;
const TILE_SELECTOR_MARGIN: f32 = 80.0;
const EDITOR_LIST_Y: f32 = MAIN_MENU_HEIGHT + 4.0;

const CURSOR_PNG: &[u8] = include_bytes!("cursor.png");

fn save(path: impl AsRef<Path>, class_editor: &mut ClassEditor) -> Result<(), Box<dyn Error>> {
	let toml = class_editor.to_toml()?;
	fs::write(path, toml)?;
	class_editor.unsaved = false;

	Ok(())
}

fn main() {
	let mut system = support::init("Furry Emblem - Editor");

	let mut selected_tile = 0;
	let mut autosave_timer = 0.0;

	let texture_atlas = register_tileset(
		system.display.get_context(),
		system.renderer.textures(),
		&image::open("example/tree_tiles.png").unwrap(),
	).unwrap();

	let cursor_tile = register_image(
		system.display.get_context(),
		system.renderer.textures(),
		&image::load_from_memory(CURSOR_PNG).unwrap(),
	).unwrap();

	// In the future, class/unit icons should be loaded from some config file.
	// Classes can be serialized in unit data as their names, since this is how users will identify them.
	let unit_icons = register_tileset(
		system.display.get_context(),
		system.renderer.textures(),
		&image::open("example/unit-icons.png").unwrap(),
	).unwrap();

	let mut item_editor = ItemEditor::new();
	let mut unit_editor = UnitEditor::new();
	let mut class_editor = ClassEditor::open("example/classes.toml", unit_icons[0]);
	let mut map = MapData::with_size(20, 20);
	let mut warning_message = String::new();

	system.main_loop(move |_, ui| {
		let display_size = ui.io().display_size;
		let save_path = "example/classes.toml";
		let ctrl = if ui.io().config_mac_os_behaviors { "Cmd" } else { "Ctrl" };
		// This is necessary to ensure that the popup is always opened from the root.
		let mut show_warning = false;

		if ui.io().key_ctrl && ui.is_key_pressed(Key::S) {
			match save(save_path, &mut class_editor) {
				Ok(_) => eprintln!("Saved class info"),
				Err(err) => {
					warning_message = format!("Failed to save class info: {err}");
					eprintln!("{warning_message}");
					show_warning = true;
				}
			}
		}

		ui.main_menu_bar(|| {
			ui.menu("File", || {
				ui.menu_item("New Project");
				ui.menu_item("New Map");
				ui.menu_item("Open Project");
				if ui.menu_item(&format!("Save ({ctrl} + S)")) {
					match save(save_path, &mut class_editor) {
						Ok(_) => eprintln!("Saved class info"),
						Err(err) => {
							warning_message = format!("Failed to save class info: {err}");
							eprintln!("{warning_message}");
							show_warning = true;
						}
					}
				}
			});
			ui.menu("View", || {
				ui.checkbox("Item editor", &mut item_editor.is_shown);
				ui.checkbox("Unit editor", &mut unit_editor.is_shown);
				ui.checkbox("Class editor", &mut class_editor.is_shown);
			});
			ui.menu_item("Options");
			ui.menu("Help", || {
				ui.text("Furry Emblem Editor");
				ui.text("By Evie M.");
			});
		});

		ui.editor_list(
			&mut item_editor,
			"Items",
			"Item",
			(MAP_VIEWER_MARGIN, EDITOR_LIST_Y),
		);

		ui.editor_list(
			&mut unit_editor,
			"Units",
			"Unit",
			(MAP_VIEWER_MARGIN + 200.0, EDITOR_LIST_Y),
		);

		ui.editor_list(
			&mut class_editor,
			"Classes",
			"Class",
			(MAP_VIEWER_MARGIN + 200.0 * 2.0, EDITOR_LIST_Y),
		);

		ui.window("Map Editor")
			.size(
				[display_size[0] - TILE_SELECTOR_MARGIN, display_size[0] - MAIN_MENU_HEIGHT],
				Condition::Always,
			)
			.position(
				[0.0, MAIN_MENU_HEIGHT],
				Condition::Always,
			)
			.movable(false)
			.bring_to_front_on_focus(false)
			.focus_on_appearing(false)
			.no_decoration()
			.build(|| {
				ui.tilemap(&mut map, &texture_atlas, &class_editor.classes, cursor_tile, selected_tile)
			});

		ui.window("Tile Selector")
			.size(
				[TILE_SELECTOR_MARGIN, display_size[0] - MAIN_MENU_HEIGHT],
				Condition::Always,
			)
			.position(
				[display_size[0] - TILE_SELECTOR_MARGIN, MAIN_MENU_HEIGHT],
				Condition::Always,
			)
			.movable(false)
			.bring_to_front_on_focus(false)
			.focus_on_appearing(false)
			.no_decoration()
			.build(|| {
				selected_tile = ui.tile_selector(
					&texture_atlas,
					selected_tile,
					cursor_tile,
				);
			});

		// End-of-frame cleanup
		item_editor.items.retain(|i| i.is_open);
		unit_editor.units.retain(|i| i.is_open);
		class_editor.classes.retain(|i| i.is_open);
		if autosave_timer > AUTOSAVE_FREQUENCY {
			if class_editor.unsaved {
				// TODO: In the future, autosaving and saving should be considered seperate actions.
				// If autosaving fails, the "autosaved" flag should still be set,
				// so that it isn't attempted again until a change is made that may fix it.
				match save("example/classes.autosave.toml", &mut class_editor) {
					Ok(_) => {
						eprintln!("Autosaved class info");
						class_editor.unsaved = false;
					}
					Err(err) => {
						eprintln!("Failed to autosave class info: {err}");
					}
				}
			}
			autosave_timer -= AUTOSAVE_FREQUENCY;
		}
		autosave_timer += ui.io().delta_time;

		if show_warning {
			ui.open_popup("Warning!");
		}

		ui.modal_popup("Warning!", || {
			ui.text(&warning_message);
			if ui.button("Ok") {
				ui.close_current_popup();
			}
		});
	});
}
