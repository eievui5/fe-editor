use crate::*;
use imgui::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MOUSE_WHEEL_ZOOM_SPEED: f32 = 3.0;
const KEYBOARD_ZOOM_SPEED: f32 = 32.0;
const KEYBOARD_DRAG_SPEED: f32 = 1024.0;

pub struct ModalCapsule {
	should_open: bool
}

impl ModalCapsule {
	pub fn new() -> Self { Self { should_open: false } }
	/// Open the popup. Equivalent to Ui::open_popup().
	pub fn open(&mut self) { self.should_open = true; }
	/// Resets the capsule state.
	pub fn reset(&mut self) { self.should_open = false; }
	/// Draw the popup. Equivalent to Ui::modal_popup().
	pub fn build<R, F: FnOnce() -> R>(
		&self,
		ui: &Ui,
		id: impl AsRef<str>,
		f: F
	) -> Option<R> {
		if self.should_open {
			ui.open_popup(&id);
		}

		ui.modal_popup(&id, f)
	}
}

pub trait CustomUi {
	fn hover_tooltip(&self, message: &str);
	fn editor_list<T: EditorList>(
		&self,
		editor: &mut T,
		title: &str,
		hint: &str,
		position: (f32, f32)
	);
	fn tilemap(
		&self,
		map: &mut MapEditor,
		texture_atlas: &Vec<TextureId>,
		classes: &Vec<ClassData>,
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

	fn editor_list<T: EditorList>(
		&self,
		editor: &mut T,
		title: &str,
		hint: &str,
		position: (f32, f32)
	) {
		let mut is_shown = *editor.is_shown();

		// Track any changes that occur during this frame.
		let mut editor_hash = DefaultHasher::new();
		editor.entries().hash(&mut editor_hash);
		let editor_hash = editor_hash.finish();

		if !is_shown {
			return;
		}

		self.window(title)
			.opened(&mut is_shown)
			.position([position.0, position.1], Condition::FirstUseEver)
			.size([200.0, 400.0], Condition::FirstUseEver)
			.menu_bar(true)
			.focus_on_appearing(false)
			.collapsed(true, Condition::FirstUseEver)
			.unsaved_document(*editor.unsaved())
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
								self.text(&format!(
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

		let mut current_hash = DefaultHasher::new();
		editor.entries().hash(&mut current_hash);
		let current_hash = current_hash.finish();

		*editor.is_shown() = is_shown;
		if !*editor.unsaved() {
			*editor.unsaved() = editor_hash != current_hash;
		}
	}

	fn tilemap(
		&self,
		map: &mut MapEditor,
		texture_atlas: &Vec<TextureId>,
		classes: &Vec<ClassData>,
		cursor_tile: TextureId,
		selected_tile: usize,
	) {
		let window_pos = self.window_pos();
		let draw_list = self.get_window_draw_list();
		let delta = self.io().delta_time;

		for ty in 0..map.height {
			for tx in 0..map.width {
				let tile = *map.get_tile(tx, ty);
				let x = (tx as f32) * map.zoom
					+ map.scroll[0]
					+ window_pos[0];
				let y = (ty as f32) * map.zoom
					+ map.scroll[1]
					+ window_pos[1];
				draw_list.add_image(
					texture_atlas[tile],
					[x, y],
					[x + map.zoom, y + map.zoom]
				).build();
			}
		}

		// Only handle input if the window is hovered.
		if self.is_window_hovered() {
			let x = (self.io().mouse_pos[0] - map.scroll[0] - window_pos[0]) / map.zoom;
			let y = (self.io().mouse_pos[1] - map.scroll[1] - window_pos[1]) / map.zoom;
			let mouse_drag_delta = self.mouse_drag_delta_with_button(MouseButton::Middle);

			let map_zoom_delta = map.zoom - (
				map.zoom
				// Enable zooming with mouse wheel...
				+ self.io().mouse_wheel * MOUSE_WHEEL_ZOOM_SPEED
				// ...as well as the - and = (meaning +) keys.
				+ if self.is_key_down(Key::Equal) { KEYBOARD_ZOOM_SPEED * delta } else { 0.0 }
				- if self.is_key_down(Key::Minus) { KEYBOARD_ZOOM_SPEED * delta } else { 0.0 }
			).clamp(16.0, 128.0);
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
			if x >= 0.0 && y >= 0.0 && x < (map.width as f32) && y < (map.height as f32) {
				if self.is_key_down(Key::MouseLeft) {
					*map.get_tile(x.floor() as usize, y.floor() as usize) = selected_tile;
				}

				if self.is_key_down(Key::MouseRight) {
					self.open_popup("info");
					map.info_popup.position = (x.floor() as u32, y.floor() as u32);
				}

				if !self.is_key_down(Key::MouseMiddle) {
					let tx = x.floor() * map.zoom
						+ map.scroll[0]
						+ window_pos[0];
					let ty = y.floor() * map.zoom
						+ map.scroll[1]
						+ window_pos[1];
					// Draw a placement preview.
					draw_list.add_image(
						texture_atlas[selected_tile],
						[tx, ty],
						[tx + map.zoom, ty + map.zoom]
					).build();
					draw_list.add_image(
						cursor_tile,
						[tx, ty],
						[tx + map.zoom, ty + map.zoom]
					).build();
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
			let mut unit_index = None;
			for (i, u) in map.units.iter_mut().enumerate() {
				if (u.x, u.y) == map.info_popup.position {
					unit = Some(u);
					unit_index = Some(i);
				}
			}

			self.popup("class menu", || {
				for (i, class) in classes.iter().enumerate() {
					// Classes per row.
					if i % 3 != 0 {
						self.same_line();
					}
					if self.image_button(
						i.to_string(),
						class.texture_id,
						[32.0; 2]
					) {
						let unit = unit
							.as_mut()
							.expect("No unit found but class popup is open");
						unit.class = i
					}
					self.hover_tooltip(&class.name);
				}
			});

			if let Some(unit) = unit {
				// Unit selected
				self.text(&format!("Class:\n{}", classes[unit.class].name));
				self.same_line();
				if self.image_button(
					"Class selector",
					classes[unit.class].texture_id,
					[32.0; 2]
				) {
					self.open_popup("class menu");
				}
				self.hover_tooltip("Click to select class");
				self.input_text("##name", &mut map.info_popup.unit)
					.hint("Name (Optional)")
					.build();
				if self.button("Delete Unit") {
					map.units.remove(
						unit_index
							.expect("No unit found but unit editor is open.")
					);
					self.close_current_popup();
				}
			} else {
				// Nothing selected
				if self.button("Place Unit") && classes.len() > 0 {
					let unit = MapUnit::at_position(
						map.info_popup.position.0,
						map.info_popup.position.1,
					);
					map.units.push(unit);
				};
				if classes.len() == 0 {
					self.hover_tooltip("Cannot create unit: No classes are defined.");
				}
				self.button("Mark as spawn");
			}

		});

		for i in &map.units {
			let x = window_pos[0] + map.scroll[0] + (i.x as f32) * map.zoom;
			let y = window_pos[1] + map.scroll[1] + (i.y as f32) * map.zoom;
			draw_list.add_image(
				classes[i.class].texture_id,
				[x, y],
				[x + map.zoom, y + map.zoom]
			).build();
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
