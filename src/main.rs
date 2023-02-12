use furry_emblem_editor::{register_texture_rgba, support};
use furry_emblem_editor::{CustomUi, Facade, ItemEditor, UnitEditor};
use imgui::*;

const MAIN_MENU_HEIGHT: f32 = 22.0;
const MAP_VIEWER_MARGIN: f32 = 32.0;
const TILE_SELECTOR_MARGIN: f32 = 128.0;
const EDITOR_LIST_Y: f32 = MAIN_MENU_HEIGHT + 4.0;
const MOUSE_WHEEL_ZOOM_SPEED: f32 = 3.0;

fn main() {
	let mut system = support::init("Furry Emblem - Editor");

	let mut selected_tile = 0;
	let texture_atlas = [
		register_texture_rgba(
			system.display.get_context(),
			system.renderer.textures(),
			{
				let mut data = Vec::with_capacity(100 * 100);
				for i in 0..100 {
					for j in 0..100 {
						// Insert RGB values
						data.push(i as u8);
						data.push(j as u8);
						data.push((i + j) as u8);
						data.push(255);
					}
				}
				data
			},
			100,
			100,
		).unwrap(),
		register_texture_rgba(
			system.display.get_context(),
			system.renderer.textures(),
			{
				let mut data = Vec::with_capacity(100 * 100);
				for i in 0..100 {
					for j in 0..100 {
						// Insert RGB values
						data.push((i + j) as u8);
						data.push(i as u8);
						data.push(j as u8);
						data.push(255);
					}
				}
				data
			},
			100,
			100,
		).unwrap()
	];

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

	let mut item_editor = ItemEditor::new();
	let mut unit_editor = UnitEditor::new();

	system.main_loop(move |_, ui| {
		let _delta = ui.io().delta_time;

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
					let x = ((ui.io().mouse_pos[0] - map_scroll[0] - window_pos[0]) / map_zoom_level) as i32;
					let y = ((ui.io().mouse_pos[1] - map_scroll[1] - window_pos[1]) / map_zoom_level) as i32;
					let mouse_drag_delta = ui.mouse_drag_delta_with_button(MouseButton::Middle);

					map_zoom_level += ui.io().mouse_wheel * MOUSE_WHEEL_ZOOM_SPEED;
					map_zoom_level = map_zoom_level.clamp(16.0, 128.0);

					if mouse_drag_delta[0] != 0.0 && mouse_drag_delta[1] != 0.0 {
						map_scroll[0] += mouse_drag_delta[0];
						map_scroll[1] += mouse_drag_delta[1];
						ui.reset_mouse_drag_delta(MouseButton::Middle);
					}

					if ui.is_key_down(Key::MouseLeft) && x >= 0 && y >= 0 {
						let x = x as usize;
						let y = y as usize;
						if x < map[0].len() && y < map.len() {
							map[y][x] = selected_tile;
						}
					}

					if !ui.is_key_down(Key::MouseMiddle) {
						let tx = (x as f32) * map_zoom_level
							+ map_scroll[0]
							+ window_pos[0];
						let ty = (y as f32) * map_zoom_level
							+ map_scroll[1]
							+ window_pos[1];
						// Draw a placement preview.
						draw_list.add_image(
							texture_atlas[selected_tile],
							[tx, ty],
							[tx + map_zoom_level, ty + map_zoom_level]
						).build();
					}
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
					if ui.image_button(i.to_string(), *texture, [64.0, 64.0]) {
						selected_tile = i;
					}
				}
			});

		// End-of-frame cleanup
		item_editor.items.retain(|i| i.is_open);
		unit_editor.units.retain(|i| i.is_open);
	});
}
