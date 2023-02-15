#![feature(path_file_prefix)]

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use furry_emblem_editor::*;
use imgui::*;

const AUTOSAVE_FREQUENCY: f32 = 2.0;
const MAIN_MENU_HEIGHT: f32 = 22.0;
const MAP_VIEWER_MARGIN: f32 = 32.0;
const TILE_SELECTOR_MARGIN: f32 = 80.0;
const EDITOR_LIST_Y: f32 = MAIN_MENU_HEIGHT + 4.0;

const CURSOR_PNG: &[u8] = include_bytes!("cursor.png");

struct NewMapPopup {
	capsule: ModalCapsule,
	width: usize,
	height: usize,
	path: String,
}

impl NewMapPopup {
	fn new() -> Self {
		Self {
			capsule: ModalCapsule::new(),
			width: 15,
			height: 10,
			path: String::new(),
		}
	}
}

fn save(
	path: impl AsRef<Path>,
	class_editor: &mut ClassEditor,
	map_editor: &mut Option<MapEditor>,
) -> Result<(), Box<dyn Error>> {
	// This seems strange...
	let path: PathBuf = [path].iter().collect();
	fs::create_dir_all(&path)?;

	let toml = class_editor.to_toml()?;
	fs::write(
		[path.clone(), "classes.toml".into()].iter().collect::<PathBuf>(),
		toml
	)?;
	class_editor.unsaved = false;

	if let Some(map_editor) = map_editor {
		let mut maps_path = PathBuf::new();
		maps_path.push(&path);
		maps_path.push("maps");
		fs::create_dir_all(&maps_path)?;
		maps_path.push(&map_editor.data.name);
		maps_path.set_extension("toml");
		let toml = map_editor.data.to_toml()?;
		fs::write(maps_path, toml)?;
	}

	Ok(())
}

fn main() {
	let mut system = support::init("Furry Emblem - Editor");

	let mut selected_tile = 0;
	let mut autosave_timer = 0.0;
	let save_path = PathBuf::from(".");
	let maps_path: PathBuf = [&*save_path.to_string_lossy(), "maps"]
		.iter()
		.collect();

	let texture_atlas = register_tileset(
		system.display.get_context(),
		system.renderer.textures(),
		&image::open("tileset.png").unwrap(),
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
		&image::open("class-icons.png").unwrap(),
	).unwrap();

	// Editors
	let mut item_editor = ItemEditor::new();
	let mut unit_editor = UnitEditor::new();
	let mut class_editor = ClassEditor::open(
		[save_path.clone(), "classes.toml".into()].iter().collect::<PathBuf>(),
		unit_icons[0]
	);
	let mut map_editor: Option<MapEditor> = None;
	// Popups
	let mut new_map_popup = NewMapPopup::new();

	let mut warning_message = String::new();
	let mut level_name = String::new();

	system.main_loop(move |_, ui| {
		let display_size = ui.io().display_size;
		let ctrl = if ui.io().config_mac_os_behaviors { "Cmd" } else { "Ctrl" };
		let mut warning_popup = ModalCapsule::new();
		let mut open_map_popup = ModalCapsule::new();
		new_map_popup.capsule.reset();
		// for the sake of not repeating save code:
		let mut manual_save = false;

		ui.main_menu_bar(|| {
			ui.menu("File", || {
				if ui.menu_item("New Map") {
					new_map_popup.capsule.open();
					level_name = String::new();
				}
				if ui.menu_item("Open Map") {
					open_map_popup.open();
					level_name = String::new();
				}
				if ui.menu_item(&format!("Save ({ctrl} + S)")) {
					manual_save = true;
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

		if let Some(mut map_editor) = map_editor.as_mut() {
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
					ui.tilemap(&mut map_editor, &texture_atlas, &class_editor.classes, cursor_tile, selected_tile)
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
		}

		// End-of-frame cleanup
		item_editor.items.retain(|i| i.is_open);
		unit_editor.units.retain(|i| i.is_open);
		class_editor.classes.retain(|i| i.is_open);
		if autosave_timer > AUTOSAVE_FREQUENCY {
			if class_editor.unsaved {
				// TODO: In the future, autosaving and saving should be considered seperate actions.
				// If autosaving fails, the "autosaved" flag should still be set,
				// so that it isn't attempted again until a change is made that may fix it.
				let mut autosave_dir = save_path.clone();
				autosave_dir.push("autosave/");
				match save(
					autosave_dir,
					&mut class_editor,
					&mut map_editor,
				) {
					Ok(_) => {
						eprintln!("Autosaved");
						class_editor.unsaved = false;
					}
					Err(err) => {
						eprintln!("Autosave failed: {err}");
					}
				}
			}
			autosave_timer -= AUTOSAVE_FREQUENCY;
		}
		autosave_timer += ui.io().delta_time;

		if manual_save || ui.io().key_ctrl && ui.is_key_pressed(Key::S) {
			match save(
				&save_path,
				&mut class_editor,
				&mut map_editor,
			) {
				Ok(_) => eprintln!("Saved"),
				Err(err) => {
					warning_message = format!("Save failed: {err}");
					eprintln!("{warning_message}");
					warning_popup.open();
				}
			}
		}

		new_map_popup.capsule.build(&ui, "New Map", || {
			ui.dummy([300.0, 0.0]);
			ui.input_text("##path", &mut new_map_popup.path)
				.hint("Name")
				.build();

			ui.text("Width:");
			ui.input_scalar("##width", &mut new_map_popup.width)
				.step(1)
				.build();

			ui.text("Height:");
			ui.input_scalar("##height", &mut new_map_popup.height)
				.step(1)
				.build();

			new_map_popup.width = new_map_popup.width.max(15);
			new_map_popup.height = new_map_popup.height.max(10);

			if ui.button("Cancel") {
				ui.close_current_popup();
			}
			ui.same_line();
			if ui.button("Create") {
				map_editor = Some(MapEditor::with_size(
					new_map_popup.path.clone(),
					new_map_popup.width,
					new_map_popup.height,
				));
				ui.close_current_popup();
			}
		});

		open_map_popup.build(&ui, "Open Map", || {
			ui.dummy([300.0, 0.0]);
			ui.text("Select a level:");
			for entry in fs::read_dir(&maps_path).unwrap().filter_map(|e| e.ok()) {
				let mut path = entry.path();
				let file_name = path.file_prefix().unwrap().to_string_lossy().to_string();
				path.pop();
				if ui.button(&file_name) {
					map_editor = Some(MapEditor::open(
						&path,
						file_name,
					).unwrap());
					ui.close_current_popup();
				}
			}
			if ui.button("Cancel") {
				ui.close_current_popup();
			}
		});

		warning_popup.build(&ui, "Warning!", || {
			ui.dummy([300.0, 0.0]);
			ui.text(&warning_message);
			if ui.button("Ok") {
				ui.close_current_popup();
			}
		});
	});
}
