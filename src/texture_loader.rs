use fe_data::FeError;
use glium::Texture2d;
use glium::backend::Facade;
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};
use image::{GenericImageView, Pixel, Rgba};
use imgui::{TextureId, Textures};
use imgui_glium_renderer::Texture;
use std::{borrow::Cow, error::Error, rc::Rc};

const TILE_SIZE: u32 = 16;

pub fn register_tileset<F, I: GenericImageView<Pixel=Rgba<u8>>>(
	gl_ctx: &F,
	textures: &mut Textures<Texture>,
	image: &I
) -> Result<Vec<TextureId>, Box<dyn Error>> where F: Facade {
	let mut ids = Vec::new();

	if image.width() % TILE_SIZE != 0 || image.height() % TILE_SIZE != 0 {
		return Err(Box::from(FeError { msg: format!(
			"Image width or height is not a multiple of {TILE_SIZE}."
		)}));
	}

	for y in (0..image.height()).step_by(TILE_SIZE as usize) {
		for x in (0..image.width()).step_by(TILE_SIZE as usize) {
			ids.push(
				register_image(
					gl_ctx,
					textures,
					&*image.view(x, y, TILE_SIZE, TILE_SIZE),
				)?
			);
		}
	}

	Ok(ids)
}

pub fn register_image<F, I: GenericImageView<Pixel=Rgba<u8>>>(
	gl_ctx: &F,
	textures: &mut Textures<Texture>,
	image: &I
) -> Result<TextureId, Box<dyn Error>> where F: Facade {
	register_texture_rgba(
		gl_ctx,
		textures,
		{
			let mut buf = Vec::new();
			for (_, _, p) in image.pixels() {
				buf.push(p.to_rgba().0[0]);
				buf.push(p.to_rgba().0[1]);
				buf.push(p.to_rgba().0[2]);
				buf.push(p.to_rgba().0[3]);
			}
			buf
		},
		image.width(),
		image.height(),
	)
}

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
			magnify_filter: MagnifySamplerFilter::Nearest,
			minify_filter: MinifySamplerFilter::Linear,
			..Default::default()
		},
	};

	Ok(textures.insert(texture))
}
