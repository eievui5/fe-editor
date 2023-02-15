use fe_data::*;
use std::error::Error;
use std::path::Path;

#[derive(Debug)]
pub struct MapEditor {
	// Data
	pub data: MapData,
	// UI fields
	pub scroll: [f32; 2],
	pub zoom: f32,
	pub info_popup: MapInfoPopup,
}

impl MapEditor {
	pub fn open(
		path: impl AsRef<Path>,
		name: String,
	) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			data: MapData::open(path, name)?,
			// UI stuff
			scroll: [0.0, 0.0],
			zoom: 64.0,
			info_popup: MapInfoPopup::new(),
		})
	}

	pub fn with_size(name: String, width: usize, height: usize) -> Self {
		Self {
			data: MapData::with_size(name, width, height),
			scroll: [0.0, 0.0],
			zoom: 64.0,
			info_popup: MapInfoPopup::new(),
		}
	}

	pub fn get_tile(&mut self, x: usize, y: usize) -> &mut usize {
		&mut self.data.data[x + y * self.data.width]
	}
}

#[derive(Debug)]
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
