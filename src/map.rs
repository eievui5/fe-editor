pub struct MapData {
	pub width: usize,
	pub height: usize,
	pub scroll: [f32; 2],
	pub zoom: f32,
	pub units: Vec<MapUnit>,
	pub info_popup: MapInfoPopup,
	data: Vec<usize>,
}

impl MapData {
	pub fn with_size(width: usize, height: usize) -> Self {
		let mut data = Vec::new();
		data.resize((width * height) as usize, 0);
		Self {
			width,
			height,
			data,
			scroll: [0.0, 0.0],
			zoom: 64.0,
			units: Vec::new(),
			info_popup: MapInfoPopup::new(),
		}
	}

	pub fn get_tile(&mut self, x: usize, y: usize) -> &mut usize {
		&mut self.data[x + y * self.width]
	}
}

/// I don't know why I made this an enum, but it's wrong and I need to fix it.
#[derive(Copy, Clone)]
pub enum UnitClass {
	Debug = 0,
}

impl UnitClass {
	pub fn as_usize(self) -> usize {
		use UnitClass::*;
		match self {
			Debug => 0,
		}
	}
}

pub struct MapInfoPopup {
	pub unit: String,
	pub position: (u32, u32),
}

impl MapInfoPopup {
	pub fn new() -> Self {
		Self {
			unit: String::new(),
			position: (0, 0),
		}
	}
}

pub struct MapUnit {
	pub x: u32,
	pub y: u32,
	pub name: String,
	pub class: UnitClass,
}

impl MapUnit {
	pub fn at_position(x: u32, y: u32) -> Self {
		Self {
			x,
			y,
			name: String::new(),
			class: UnitClass::Debug,
		}
	}
}
