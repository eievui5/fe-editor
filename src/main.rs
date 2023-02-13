use furry_emblem_editor::*;
use imgui::*;

const MAIN_MENU_HEIGHT: f32 = 22.0;
const MAP_VIEWER_MARGIN: f32 = 32.0;
const TILE_SELECTOR_MARGIN: f32 = 128.0;
const EDITOR_LIST_Y: f32 = MAIN_MENU_HEIGHT + 4.0;
const MOUSE_WHEEL_ZOOM_SPEED: f32 = 3.0;
const KEYBOARD_ZOOM_SPEED: f32 = 32.0;
const KEYBOARD_DRAG_SPEED: f32 = 1024.0;

const CURSOR_PNG: &[u8] = include_bytes!("cursor.png");

struct MapInfoPopup {
	position: (u32, u32),
}

impl MapInfoPopup {
	fn new() -> Self {
		Self {
			position: (0, 0),
		}
	}
}

fn main() {
	let mut system = support::init("Furry Emblem - Editor");

	let mut selected_tile = 0;
	let texture_atlas = register_tileset(
		system.display.get_context(),
		system.renderer.textures(),
		&image::open("src/tree_tiles.png").unwrap(),
	).unwrap();

	let highlight_tile = register_image(
		system.display.get_context(),
		system.renderer.textures(),
		&image::load_from_memory(CURSOR_PNG).unwrap(),
	).unwrap();

	let mut map_zoom_level: f32 = 64.0;
	let mut map_scroll: [f32; 2] = [0.0, 0.0];
	let mut map: Vec<Vec<usize>> = vec![
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
	];
	let mut unit_list = Vec::<(u32, u32)>::new();
	let mut info_popup = MapInfoPopup::new();

	let mut item_editor = ItemEditor::new();
	let mut unit_editor = UnitEditor::new();

	system.main_loop(move |_, ui| {
		let delta = ui.io().delta_time;

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

		let display_size = ui.io().display_size;

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
				let window_pos = ui.window_pos();
				let draw_list = ui.get_window_draw_list();
				for (ty, row) in map.iter().enumerate() {
					for (tx, tile) in row.iter().enumerate() {
						let x = (tx as f32) * map_zoom_level
							+ map_scroll[0]
							+ window_pos[0];
						let y = (ty as f32) * map_zoom_level
							+ map_scroll[1]
							+ window_pos[1];
						draw_list.add_image(
							texture_atlas[*tile],
							[x, y],
							[x + map_zoom_level, y + map_zoom_level]
						).build();
					}
				}

				// Only handle input if the window is hovered.
				if ui.is_window_hovered() {
					let x = (ui.io().mouse_pos[0] - map_scroll[0] - window_pos[0]) / map_zoom_level;
					let y = (ui.io().mouse_pos[1] - map_scroll[1] - window_pos[1]) / map_zoom_level;
					let mouse_drag_delta = ui.mouse_drag_delta_with_button(MouseButton::Middle);

					let map_zoom_delta = map_zoom_level - (
						map_zoom_level
						// Enable zooming with mouse wheel...
						+ ui.io().mouse_wheel * MOUSE_WHEEL_ZOOM_SPEED
						// ...as well as the - and = (meaning +) keys.
						+ if ui.is_key_down(Key::Equal) { KEYBOARD_ZOOM_SPEED * delta } else { 0.0 }
						- if ui.is_key_down(Key::Minus) { KEYBOARD_ZOOM_SPEED * delta } else { 0.0 }
					).clamp(16.0, 128.0);
					map_zoom_level = map_zoom_level - map_zoom_delta;

					map_scroll[0] += x * map_zoom_delta;
					map_scroll[1] += y * map_zoom_delta;
					if ui.is_key_down(Key::LeftArrow) {
						map_scroll[0] += KEYBOARD_DRAG_SPEED * delta;
					}
					if ui.is_key_down(Key::RightArrow) {
						map_scroll[0] -= KEYBOARD_DRAG_SPEED * delta;
					}
					if ui.is_key_down(Key::UpArrow) {
						map_scroll[1] += KEYBOARD_DRAG_SPEED * delta;
					}
					if ui.is_key_down(Key::DownArrow) {
						map_scroll[1] -= KEYBOARD_DRAG_SPEED * delta;
					}

					if mouse_drag_delta[0] != 0.0 && mouse_drag_delta[1] != 0.0 {
						map_scroll[0] += mouse_drag_delta[0];
						map_scroll[1] += mouse_drag_delta[1];
						ui.reset_mouse_drag_delta(MouseButton::Middle);
					}

					// Only if the cursor is over the map.
					if x >= 0.0 && y >= 0.0 && x < (map[0].len() as f32) && y < (map.len() as f32) {
						if ui.is_key_down(Key::MouseLeft) {
							map[y as usize][x as usize] = selected_tile;
						}

						if ui.is_key_down(Key::MouseRight) {
							ui.open_popup("info");
							info_popup.position = (x.floor() as u32, y.floor() as u32);
						}

						if !ui.is_key_down(Key::MouseMiddle) {
							let tx = x.floor() * map_zoom_level
								+ map_scroll[0]
								+ window_pos[0];
							let ty = y.floor() * map_zoom_level
								+ map_scroll[1]
								+ window_pos[1];
							// Draw a placement preview.
							draw_list.add_image(
								texture_atlas[selected_tile],
								[tx, ty],
								[tx + map_zoom_level, ty + map_zoom_level]
							).build();
							draw_list.add_image(
								highlight_tile,
								[tx, ty],
								[tx + map_zoom_level, ty + map_zoom_level]
							).build();
						}
					}
				}

				ui.popup("info", || {
					if ui.button("Close") {
						ui.close_current_popup();
					}
					if ui.button("Place Unit") {
						unit_list.push(info_popup.position);
					};
					ui.button("Mark as spawn");
				});

				for i in &unit_list {
					draw_list.add_image(
						highlight_tile,
						[i.0 as f32, i.1 as f32],
						[(i.0 + 16) as f32, (i.1 + 16) as f32]
					).build();
				}
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
				for (i, texture) in texture_atlas.iter().enumerate() {
					if ui.invisible_button(i.to_string(), [64.0, 64.0]) {
						selected_tile = i;
					}
					let draw_list = ui.get_window_draw_list();
					draw_list
						.add_image(*texture, ui.item_rect_min(), ui.item_rect_max())
						.build();
					if selected_tile == i {
						draw_list
							.add_image(highlight_tile, ui.item_rect_min(), ui.item_rect_max())
							.build();
					}
				}
			});

		// End-of-frame cleanup
		item_editor.items.retain(|i| i.is_open);
		unit_editor.units.retain(|i| i.is_open);
	});
}
