use furry_emblem_editor::*;
use imgui::*;

const MAIN_MENU_HEIGHT: f32 = 22.0;
const MAP_VIEWER_MARGIN: f32 = 32.0;
const TILE_SELECTOR_MARGIN: f32 = 80.0;
const EDITOR_LIST_Y: f32 = MAIN_MENU_HEIGHT + 4.0;

const CURSOR_PNG: &[u8] = include_bytes!("cursor.png");

fn main() {
	let mut system = support::init("Furry Emblem - Editor");

	let mut selected_tile = 0;

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

	let unit_icons = register_tileset(
		system.display.get_context(),
		system.renderer.textures(),
		&image::open("example/unit-icons.png").unwrap(),
	).unwrap();

	let mut item_editor = ItemEditor::new();
	let mut unit_editor = UnitEditor::new();
	let mut map = MapData::with_size(15, 10);

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
			ui.menu("View", || {
				ui.checkbox("Item editor", &mut item_editor.is_shown);
				ui.checkbox("Unit editor", &mut unit_editor.is_shown);
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
			(MAP_VIEWER_MARGIN, EDITOR_LIST_Y),
		);

		ui.editor_list(
			&mut unit_editor,
			"Unit",
			(MAP_VIEWER_MARGIN + 200.0, EDITOR_LIST_Y),
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
				ui.tilemap(&mut map, &texture_atlas, &unit_icons, cursor_tile, selected_tile)
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
	});
}
