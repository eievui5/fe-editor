use crate::ListItem;
use crate::EditorList;
use imgui::*;

pub trait CustomUi {
	fn hover_tooltip(&self, message: &str);
	fn editor_list<T: EditorList>(&self, editor: &mut T, hint: &str, position: (f32, f32));
}

impl CustomUi for Ui {
	fn hover_tooltip(&self, message: &str) {
		if self.is_item_hovered_with_flags(ItemHoveredFlags::DELAY_NORMAL) {
			self.tooltip(|| {
				self.text(message);
			});
		}
	}

	fn editor_list<T: EditorList>(&self, editor: &mut T, hint: &str, position: (f32, f32)) {
		let mut is_shown = *editor.is_shown();

		if !is_shown {
			return;
		}

		self.window(hint)
			.opened(&mut is_shown)
			.position([position.0, position.1], Condition::FirstUseEver)
			.size([200.0, 400.0], Condition::FirstUseEver)
			.menu_bar(true)
			.focus_on_appearing(false)
			.unsaved_document(editor.has_changes())
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
								self.text_wrapped(&format!(
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

		*editor.is_shown() = is_shown;
	}
}
