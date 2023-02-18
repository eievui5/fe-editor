use crate::*;
use imgui::*;
use imgui::color::ImColor32;

const MOUSE_WHEEL_ZOOM_SPEED: f32 = 3.0;
const KEYBOARD_ZOOM_SPEED: f32 = 32.0;
const KEYBOARD_DRAG_SPEED: f32 = 1024.0;

pub struct ModalCapsule {
	should_open: bool,
}

impl ModalCapsule {
	pub fn new() -> Self {
		Self { should_open: false }
	}
	/// Open the popup. Equivalent to Ui::open_popup().
	pub fn open(&mut self) {
		self.should_open = true;
	}
	/// Resets the capsule state.
	pub fn reset(&mut self) {
		self.should_open = false;
	}
	/// Draw the popup. Equivalent to Ui::modal_popup().
	pub fn build<R, F: FnOnce() -> R>(&self, ui: &Ui, id: impl AsRef<str>, f: F) -> Option<R> {
		if self.should_open {
			ui.open_popup(&id);
		}

		ui.modal_popup(&id, f)
	}
}

pub trait CustomUi {
	fn hover_tooltip(&self, message: &str);
	fn tilemap(
		&self,
		map: &mut MapEditor,
		texture_atlas: &Vec<TextureId>,
		classes: &Vec<ClassEntry>,
		class_icons: &ClassIcons,
		cursor_tile: TextureId,
		selected_tile: usize,
	);
	fn tile_selector(
		&self,
		texture_atlas: &Vec<TextureId>,
		selected_tile: usize,
		highlight_tile: TextureId,
	) -> usize;
}

impl CustomUi for Ui {
	fn hover_tooltip(&self, message: &str) {
		if self.is_item_hovered_with_flags(ItemHoveredFlags::DELAY_NORMAL) {
			self.tooltip(|| {
				self.text(message);
			});
		}
	}

	fn tilemap(
		&self,
		map: &mut MapEditor,
		texture_atlas: &Vec<TextureId>,
		classes: &Vec<ClassEntry>,
		class_icons: &ClassIcons,
		cursor_tile: TextureId,
		selected_tile: usize,
	) {
		let window_pos = self.window_pos();
		let draw_list = self.get_window_draw_list();
		let delta = self.io().delta_time;

		for ty in 0..map.data.height {
			for tx in 0..map.data.width {
				let tile = *map.get_tile(tx, ty);
				let x = (tx as f32) * map.zoom + map.scroll[0] + window_pos[0];
				let y = (ty as f32) * map.zoom + map.scroll[1] + window_pos[1];
				draw_list
					.add_image(texture_atlas[tile], [x, y], [x + map.zoom, y + map.zoom])
					.build();
			}
		}

		// Only handle input if the window is hovered.
		if self.is_window_hovered() {
			let x = (self.io().mouse_pos[0] - map.scroll[0] - window_pos[0]) / map.zoom;
			let y = (self.io().mouse_pos[1] - map.scroll[1] - window_pos[1]) / map.zoom;
			let mouse_drag_delta = self.mouse_drag_delta_with_button(MouseButton::Middle);

			let map_zoom_delta = map.zoom
				- (map.zoom
				// Enable zooming with mouse wheel...
				+ self.io().mouse_wheel * MOUSE_WHEEL_ZOOM_SPEED
				// ...as well as the - and = (meaning +) keys.
				+ if self.is_key_down(Key::Equal) { KEYBOARD_ZOOM_SPEED * delta } else { 0.0 }
					- if self.is_key_down(Key::Minus) {
						KEYBOARD_ZOOM_SPEED * delta
					} else {
						0.0
					})
				.clamp(16.0, 128.0);
			map.zoom = map.zoom - map_zoom_delta;

			map.scroll[0] += x * map_zoom_delta;
			map.scroll[1] += y * map_zoom_delta;
			if self.is_key_down(Key::LeftArrow) {
				map.scroll[0] += KEYBOARD_DRAG_SPEED * delta;
			}
			if self.is_key_down(Key::RightArrow) {
				map.scroll[0] -= KEYBOARD_DRAG_SPEED * delta;
			}
			if self.is_key_down(Key::UpArrow) {
				map.scroll[1] += KEYBOARD_DRAG_SPEED * delta;
			}
			if self.is_key_down(Key::DownArrow) {
				map.scroll[1] -= KEYBOARD_DRAG_SPEED * delta;
			}

			if mouse_drag_delta[0] != 0.0 && mouse_drag_delta[1] != 0.0 {
				map.scroll[0] += mouse_drag_delta[0];
				map.scroll[1] += mouse_drag_delta[1];
				self.reset_mouse_drag_delta(MouseButton::Middle);
			}

			// Only if the cursor is over the map.
			if x >= 0.0 && y >= 0.0 && x < (map.data.width as f32) && y < (map.data.height as f32) {
				if self.is_key_down(Key::MouseLeft) {
					*map.get_tile(x.floor() as usize, y.floor() as usize) = selected_tile;
				}

				if self.is_key_down(Key::MouseRight) {
					self.open_popup("info");
					map.info_popup.position = (x.floor() as u32, y.floor() as u32);
				}

				if !self.is_key_down(Key::MouseMiddle) {
					let tx = x.floor() * map.zoom + map.scroll[0] + window_pos[0];
					let ty = y.floor() * map.zoom + map.scroll[1] + window_pos[1];
					// Draw a placement preview.
					draw_list
						.add_image(
							texture_atlas[selected_tile],
							[tx, ty],
							[tx + map.zoom, ty + map.zoom],
						)
						.build();
					draw_list
						.add_image(cursor_tile, [tx, ty], [tx + map.zoom, ty + map.zoom])
						.build();
				}
			}
		}

		self.popup("info", || {
			self.text("Tile Attributes");
			if self.button("Close") {
				self.close_current_popup();
			}
			self.separator();

			let mut unit = None;
			// This index is needed to remove the unit upon deletion.
			let mut unit_index = 0;
			for (i, u) in map.data.units.iter_mut().enumerate() {
				if (u.x, u.y) == map.info_popup.position {
					unit = Some(u);
					unit_index = i;
				}
			}

			let mut spawnpoint_index = None;
			for (i, u) in map.data.spawns.iter_mut().enumerate() {
				if *u == map.info_popup.position {
					spawnpoint_index = Some(i);
				}
			}

			self.popup("class menu", || {
				for (i, class) in classes.iter().enumerate() {
					// Classes per row.
					if i % 3 != 0 {
						self.same_line();
					}
					if self.image_button(i.to_string(), class_icons[&class.data.texture], [32.0; 2]) {
						let unit = unit
							.as_mut()
							.expect("No unit found but class popup is open");
						unit.class = i
					}
					self.hover_tooltip(&class.data.name);
				}
			});

			if let Some(unit) = unit {
				// Unit selected
				self.text(&format!("Class:\n{}", classes[unit.class].data.name));
				self.same_line();
				if self.image_button(
					"Class selector",
					class_icons[&classes[unit.class].data.texture],
					[32.0; 2],
				) {
					self.open_popup("class menu");
				}
				self.hover_tooltip("Click to select class");
				self.input_text("##name", &mut unit.name)
					.hint("Name (Optional)")
					.build();
				if self.button("Delete Unit") {
					map.data.units.remove(unit_index);
					self.close_current_popup();
				}
			} else if let Some(spawnpoint_index) = spawnpoint_index {
				// Spawnpoint selected
				if self.button("Delete Spawn") {
					map.data.spawns.remove(spawnpoint_index);
					self.close_current_popup();
				}
			} else {
				// Nothing selected
				if self.button("Place Unit") && classes.len() > 0 {
					let unit =
						MapUnit::at_position(map.info_popup.position.0, map.info_popup.position.1);
					map.data.units.push(unit);
				};
				if classes.len() == 0 {
					self.hover_tooltip("Cannot create unit: No classes are defined.");
				}
				if self.button("Mark as spawn") {
					map.data.spawns.push(map.info_popup.position);
					self.close_current_popup();
				}
			}
		});

		for i in &map.data.units {
			let x = window_pos[0] + map.scroll[0] + (i.x as f32) * map.zoom;
			let y = window_pos[1] + map.scroll[1] + (i.y as f32) * map.zoom;
			draw_list
				.add_image(
					class_icons[&classes[i.class].data.texture],
					[x, y],
					[x + map.zoom, y + map.zoom],
				)
				.build();
			if i.name.len() > 0 {
				const NAME_MARGIN: f32 = 4.0;
				let size = self.calc_text_size(&i.name);
				let center = x + map.zoom / 2.0 - size[0] / 2.0;
				draw_list
					.add_rect(
						[center - NAME_MARGIN, y - size[1] - NAME_MARGIN],
						[center + size[0] + NAME_MARGIN, y + NAME_MARGIN],
						ImColor32::from_rgb(20, 20, 20)
					)
					.filled(true)
					.rounding(5.0)
					.build();
				draw_list.add_text([center, y - size[1]], ImColor32::WHITE, &i.name);
			}
		}

		for i in &map.data.spawns {
			let x = window_pos[0] + map.scroll[0] + (i.0 as f32) * map.zoom;
			let y = window_pos[1] + map.scroll[1] + (i.1 as f32) * map.zoom;
			draw_list
				.add_image(cursor_tile, [x, y], [x + map.zoom, y + map.zoom])
				.build();
		}
	}

	fn tile_selector(
		&self,
		texture_atlas: &Vec<TextureId>,
		mut selected_tile: usize,
		highlight_tile: TextureId,
	) -> usize {
		for (i, texture) in texture_atlas.iter().enumerate() {
			if self.invisible_button(i.to_string(), [64.0, 64.0]) {
				selected_tile = i;
			}
			let draw_list = self.get_window_draw_list();
			draw_list
				.add_image(*texture, self.item_rect_min(), self.item_rect_max())
				.build();
			if selected_tile == i {
				draw_list
					.add_image(highlight_tile, self.item_rect_min(), self.item_rect_max())
					.build();
			}
		}

		selected_tile
	}
}
