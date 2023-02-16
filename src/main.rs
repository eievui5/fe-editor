#![feature(path_file_prefix)]

use fe_editor::*;
use imgui::*;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use toml::*;

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

struct EditorConfig {
	save_path: PathBuf,
}

impl EditorConfig {
	fn open() -> Result<Self, Box<dyn Error>> {
		let args: Vec<String> = env::args().collect();

		let config_path = if args.len() == 1 {
			"fe-editor.toml"
		} else if args.len() == 2 {
			&args[1]
		} else {
			Err(FeError::from(format!(
				"Too many args. Usage: {} [config file]",
				args[0]
			)))?
		};

		let mut config = EditorConfig {
			save_path: PathBuf::from("."),
		};

		match fs::read_to_string(config_path) {
			Ok(text) => {
				let toml: Table = text.parse()?;
				for (key, value) in toml {
					match key.as_str() {
						"project" => {
							if let Value::String(value) = value {
								config.save_path = value.into();
							} else {
								eprintln!("Failed to read project path: not a string");
							}
						}
						_ => {
							eprintln!("Unrecognized key: {key}");
						}
					}
				}
			}
			Err(msg) => {
				eprintln!("Failed to load config file: {msg}. Treating current directory as project root.");
			}
		}

		Ok(config)
	}
}

/// Creates and iterates over a directory.
fn walk_directory(
	path: impl AsRef<Path>,
	mut f: impl FnMut(fs::DirEntry) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
	if let Err(msg) = fs::create_dir_all(&path) {
		Err(FeError::from(format!(
			"Cannot create level directory: {msg}"
		)))?
	}

	// Now try to iterate over it.
	let dir = match fs::read_dir(&path) {
		Ok(dir) => dir,
		Err(msg) => Err(FeError::from(format!("Cannot load level list: {msg}")))?,
	};
	for entry in dir.filter_map(|e| e.ok()) {
		f(entry)?;
	}

	Ok(())
}

fn append_path(path: &PathBuf, s: &str) -> PathBuf {
	[&*path.to_string_lossy(), s].iter().collect()
}

fn save(
	path: PathBuf,
	class_editor: &mut ClassEditor,
	map_editor: &mut Option<MapEditor>,
) -> Result<(), Box<dyn Error>> {
	fs::create_dir_all(&path)?;

	let toml = class_editor.to_toml()?;
	fs::write(append_path(&path, "classes.toml"), toml)?;
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

fn main() -> Result<(), Box<dyn Error>> {
	let mut system = support::init("Furry Emblem - Editor");
	let config = EditorConfig::open()?;
	let maps_path: PathBuf = append_path(&config.save_path, "maps");
	let unit_icons_path: PathBuf = append_path(&config.save_path, "class-icons");

	let mut selected_tile = 0;
	let mut autosave_timer = 0.0;

	let cursor_tile = register_image(
		system.display.get_context(),
		system.renderer.textures(),
		// This unwrap is safe; CURSOR_PNG is constant.
		&image::load_from_memory(CURSOR_PNG).unwrap(),
	)
	.unwrap();

	let texture_atlas = register_tileset(
		system.display.get_context(),
		system.renderer.textures(),
		&image::open(append_path(&config.save_path, "tileset.png"))?,
	)
	.unwrap();

	// In the future, class/unit icons should be loaded from some config file.
	// Classes can be serialized in unit data as their names, since this is how users will identify them.
	let mut default_class_icon = None;
	let mut unit_icons = ClassIcons::new();

	if let Err(msg) = walk_directory(&unit_icons_path, |entry| {
		// Set the default class to whatever we find first.
		default_class_icon = Some(entry.path());
		unit_icons.insert(
			entry.path(),
			register_image(
				system.display.get_context(),
				system.renderer.textures(),
				&image::open(entry.path())?,
			)?,
		);
		Ok(())
	}) {
		eprintln!("Failed to load unit icons: {msg}");
	}

	if unit_icons.len() == 0 {
		eprintln!("No unit icons are loaded. Exiting.");
		exit(1);
	}

	// Editors
	let mut class_editor = ClassEditor::open(
		append_path(&config.save_path, "classes.toml"),
		// This is safe to unwrap.
		default_class_icon.unwrap(),
	);
	let mut map_editor: Option<MapEditor> = None;

	// Popups
	let mut new_map_popup = NewMapPopup::new();
	let mut warning_message = String::new();
	let mut level_name = String::new();

	system.main_loop(move |_, ui| {
		let display_size = ui.io().display_size;

		let (ctrl, ctrl_str) = if ui.io().config_mac_os_behaviors {
			(ui.io().key_super, "Cmd")
		} else {
			(ui.io().key_ctrl, "Ctrl")
		};
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
				if ui.menu_item(&format!("Save ({ctrl_str} + S)")) {
					manual_save = true;
				}
			});
			ui.menu("Info", || {
				ui.text("Furry Emblem Editor");
				ui.text("By Evie M.");
			});
		});

		class_editor.draw(
			&ui,
			(MAP_VIEWER_MARGIN + 200.0 * 0.0, EDITOR_LIST_Y),
			&unit_icons,
		);

		if let Some(mut map_editor) = map_editor.as_mut() {
			ui.window("Map Editor")
				.size(
					[
						display_size[0] - TILE_SELECTOR_MARGIN,
						display_size[0] - MAIN_MENU_HEIGHT,
					],
					Condition::Always,
				)
				.position([0.0, MAIN_MENU_HEIGHT], Condition::Always)
				.movable(false)
				.bring_to_front_on_focus(false)
				.focus_on_appearing(false)
				.no_decoration()
				.build(|| {
					ui.tilemap(
						&mut map_editor,
						&texture_atlas,
						&class_editor.classes,
						&unit_icons,
						cursor_tile,
						selected_tile,
					)
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
					selected_tile = ui.tile_selector(&texture_atlas, selected_tile, cursor_tile);
				});
		}

		// End-of-frame cleanup
		class_editor.classes.retain(|i| i.is_open);
		if autosave_timer > AUTOSAVE_FREQUENCY {
			if class_editor.unsaved {
				// TODO: In the future, autosaving and saving should be considered seperate actions.
				// If autosaving fails, the "autosaved" flag should still be set,
				// so that it isn't attempted again until a change is made that may fix it.
				match save(
					append_path(&config.save_path, "autosave/"),
					&mut class_editor,
					&mut map_editor,
				) {
					Ok(_) => eprintln!("Autosaved"),
					Err(err) => eprintln!("Autosave failed: {err}"),
				}
			}
			autosave_timer -= AUTOSAVE_FREQUENCY;
		}
		autosave_timer += ui.io().delta_time;

		if manual_save || ctrl && ui.is_key_pressed(Key::S) {
			match save(config.save_path.clone(), &mut class_editor, &mut map_editor) {
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
			if new_map_popup.path.len() == 0 {
				ui.button("Create");
				ui.hover_tooltip("Level must have a name");
			} else {
				if ui.button("Create") {
					map_editor = Some(MapEditor::with_size(
						new_map_popup.path.clone(),
						new_map_popup.width,
						new_map_popup.height,
					));
					ui.close_current_popup();
				}
			}
		});

		open_map_popup.build(&ui, "Open Map", || {
			ui.dummy([300.0, 0.0]);
			ui.text("Select a level:");

			if let Err(msg) = walk_directory(&maps_path, |entry| {
				let mut path = entry.path();
				// Since `maps_path` is garuanteed to be a path, this unwrap is safe.
				let file_name = path.file_prefix().unwrap().to_string_lossy().to_string();
				path.pop();
				if ui.button(&file_name) {
					match MapEditor::open(&path, file_name) {
						Ok(editor) => map_editor = Some(editor),
						Err(msg) => {
							warning_message = format!("Cannot load level: {msg}");
							warning_popup.open();
						}
					}
					ui.close_current_popup();
				}
				Ok(())
			}) {
				warning_message = format!("Failed to read levels: {msg}");
				warning_popup.open();
				ui.close_current_popup();
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
	Ok(())
}
