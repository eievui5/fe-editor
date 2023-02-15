use fe_data::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let level = MapData::open("../example/maps", "Debug Map".to_string())?;
	println!("{}", level.to_engine()?);
	Ok(())
}
