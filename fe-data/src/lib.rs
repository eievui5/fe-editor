mod map;

pub use map::*;

use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct FeError {
	pub msg: String,
}

impl Display for FeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		write!(f, "{}", self.msg)
	}
}

impl From<String> for FeError {
	fn from(s: String) -> Self {
		Self {
			msg: s,
		}
	}
}

impl From<&str> for FeError {
	fn from(s: &str) -> Self {
		Self {
			msg: s.to_string(),
		}
	}
}

impl std::error::Error for FeError {}
