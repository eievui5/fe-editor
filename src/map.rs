use crate::Error as FeError;
use std::fmt::Write;
use std::fs;
use std::error::Error;
use std::path::{Path, PathBuf};
use toml::*;

pub struct MapEditor {
	// Data
	pub name: String,
	pub width: usize,
	pub height: usize,
	data: Vec<usize>,
	pub units: Vec<MapUnit>,
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
		macro_rules! unwrap_toml {
			($table:ident, $key:literal, $variant:ident, $type:literal) => {
				{
					let value = $table
						.get($key)
						.ok_or(FeError::from(concat!("`", $key, "` not found")))?;
					if let Value::$variant(value) = value {
						value
					} else {
						Err(FeError::from(concat!("`", $key, "` must be an ", $type)))?
					}
				}
			};

			($table:ident[$key:literal] as Integer) => {
				unwrap_toml!($table, $key, Integer, "integer")
			};

			($table:ident[$key:literal] as Array) => {
				unwrap_toml!($table, $key, Array, "array")
			};
		}

		let mut map_path = PathBuf::new();
		map_path.push(path);
		map_path.push(&name);
		map_path.set_extension("toml");
		let toml = fs::read_to_string(&map_path).map_err(|err| {
			FeError::from(format!("Failed to open {}: {err}", map_path.display()))
		})?;
		let table: Table = toml.parse()?;

		let width = *unwrap_toml!(table["width"] as Integer) as usize;
		let height = *unwrap_toml!(table["height"] as Integer) as usize;

		let mut data = Vec::new();
		for i in unwrap_toml!(table["data"] as Array) {
			if let Value::Integer(id) = i {
				data.push(*id as usize);
			}
		}

		Ok(Self {
			name,
			width,
			height,
			data,
			// TODO: load this
			units: Vec::new(),
			// UI stuff
			scroll: [0.0, 0.0],
			zoom: 64.0,
			info_popup: MapInfoPopup::new(),
		})
	}

	pub fn with_size(name: String, width: usize, height: usize) -> Self {
		let mut data = Vec::new();
		data.resize((width * height) as usize, 0);
		Self {
			name,
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

	pub fn to_toml(&self) -> Result<String, Box<dyn Error>> {
		let mut toml = String::new();
		writeln!(toml, "width = {}", self.width)?;
		writeln!(toml, "height = {}", self.height)?;
		writeln!(toml, "data = {:?}", self.data)?;
		Ok(toml)
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
	pub class: usize,
}

impl MapUnit {
	pub fn at_position(x: u32, y: u32) -> Self {
		Self {
			x,
			y,
			name: String::new(),
			class: 0,
		}
	}
}
