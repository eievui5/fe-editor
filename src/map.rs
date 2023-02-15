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
	pub spawns: Vec<(u32, u32)>,
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

			($table:ident[$key:literal] as String) => {
				unwrap_toml!($table, $key, String, "string")
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
			} else {
				Err(FeError::from(format!(
					"Failed to read {}: non-integer value in `data`",
					map_path.display()
				)))?
			}
		}

		let mut units = Vec::new();
		for i in unwrap_toml!(table["units"] as Array) {
			if let Value::Table(i) = i {
				let unit = MapUnit {
					x: *unwrap_toml!(i["x"] as Integer) as u32,
					y: *unwrap_toml!(i["y"] as Integer) as u32,
					name: unwrap_toml!(i["name"] as String).clone(),
					class: *unwrap_toml!(i["class"] as Integer) as usize,
				};
				units.push(unit);
			} else {
				Err(FeError::from(format!(
					"Failed to read {}: non-dict value in `units`",
					map_path.display()
				)))?
			}
		}

		let mut spawns = Vec::new();
		for i in unwrap_toml!(table["spawns"] as Array) {
			if let Value::Table(i) = i {
				spawns.push((
					*unwrap_toml!(i["x"] as Integer) as u32,
					*unwrap_toml!(i["y"] as Integer) as u32,
				));
			} else {
				Err(FeError::from(format!(
					"Failed to read {}: non-dict value in `spawns`",
					map_path.display()
				)))?
			}
		}

		Ok(Self {
			name,
			width,
			height,
			data,
			units,
			spawns,
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
			units: Vec::new(),
			spawns: Vec::new(),
			scroll: [0.0, 0.0],
			zoom: 64.0,
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

		writeln!(toml, "units = [")?;
		for i in &self.units {
			writeln!(toml, "\t{{{}}},", i.to_toml_dict()?)?;
		}
		writeln!(toml, "]")?;

		writeln!(toml, "spawns = [")?;
		for i in &self.spawns {
			writeln!(toml, "\t{{x = {}, y = {}}}", i.0, i.1)?;
		}
		writeln!(toml, "]")?;

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

	pub fn to_toml_dict(&self) -> Result<String, Box<dyn Error>> {
		let mut toml = String::new();
		write!(toml, "x = {}, ", self.x)?;
		write!(toml, "y = {}, ", self.y)?;
		write!(toml, "name = {:?}, ", self.name)?;
		// MAKE SURE LAST FIELD HAS NO COMMA!!!
		write!(toml, "class = {}", self.class)?;
		Ok(toml)
	}
}
