use furry_emblem_editor::support;
use furry_emblem_editor::{CustomUi, ItemEditor, UnitEditor};

const MAIN_MENU_HEIGHT: f32 = 22.0;
const MAP_VIEWER_MARGIN: f32 = 32.0;
const MAP_BLANK_COLOR: [f32; 3] = [0.50, 0.50, 0.75];

fn main() {
	let system = support::init("Furry Emblem - Editor");

	let mut item_editor = ItemEditor::new();
	let mut unit_editor = UnitEditor::new();

	system.main_loop(move |_, ui| {
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
			(32.0, 32.0),
		);

		ui.editor_list(
			&mut unit_editor,
			"Unit",
			(32.0 * 2.0 + 200.0, 32.0),
		);

		let display_size = ui.io().display_size;
		let draw_list = ui.get_background_draw_list();
		// Draw a line
		draw_list.add_rect(
			[MAP_VIEWER_MARGIN, MAIN_MENU_HEIGHT + MAP_VIEWER_MARGIN],
			[display_size[0] - MAP_VIEWER_MARGIN, display_size[1] - MAP_VIEWER_MARGIN],
			MAP_BLANK_COLOR,
		)
			.filled(true)
			.rounding(2.0)
			.build();

		// End-of-frame cleanup
		item_editor.items.retain(|i| i.is_open);
		unit_editor.units.retain(|i| i.is_open);
	});
}
