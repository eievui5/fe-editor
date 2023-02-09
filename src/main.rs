use imgui::*;

mod support;

const WINDOW_NAME: &str = "Furry Emblem - Editor";

fn main() {
	let system = support::init(WINDOW_NAME);

	let mut value = 0;
	let choices = ["Hide Window", "Show Window"];

	system.main_loop(move |_, ui| {
		ui.window(WINDOW_NAME)
			.size(ui.io().display_size, Condition::Always)
			.position([0.0, 0.0], Condition::Always)
			.no_decoration()
			.build(|| {
				if value == 0 {
					ui.text_wrapped("Hello world!");
					if ui.button(choices[value]) {
						value += 1;
						value %= 2;
					}
					ui.separator();
					let mouse_pos = ui.io().mouse_pos;
					ui.text(format!(
						"Mouse Position: ({:.1},{:.1})",
						mouse_pos[0], mouse_pos[1]
					));
				} else {
					if ui.button(choices[value]) {
						value += 1;
						value %= 2;
					}
				}
			});
	});
}
