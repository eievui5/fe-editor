use imgui::TextureId;
use imgui::Textures;
use imgui_glium_renderer::Texture;
use std::borrow::Cow;
use std::error::Error;
use std::rc::Rc;
use glium::Texture2d;
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};
use glium::backend::Facade;

pub fn register_texture_rgba<F>(
	gl_ctx: &F,
	textures: &mut Textures<Texture>,
	data: Vec<u8>,
	width: u32,
	height: u32,
) -> Result<TextureId, Box<dyn Error>> where F: Facade {
	let raw = RawImage2d {
		data: Cow::Owned(data),
		width,
		height,
		format: ClientFormat::U8U8U8U8,
	};
	let gl_texture = Texture2d::new(gl_ctx, raw)?;
	let texture = Texture {
		texture: Rc::new(gl_texture),
		sampler: SamplerBehavior {
			magnify_filter: MagnifySamplerFilter::Linear,
			minify_filter: MinifySamplerFilter::Linear,
			..Default::default()
		},
	};

	Ok(textures.insert(texture))
}
