use bevy::{prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, render::render_asset::RenderAssetUsages};

#[derive(Resource)]
pub struct TextureAssets {
    pub dirt: [Handle<Image>; 4],
    pub stone: [Handle<Image>; 4],
    pub sand: [Handle<Image>; 4],
    pub grass: [Handle<Image>; 4],
    pub wet_dirt: [Handle<Image>; 4],
    pub loose_dirt: [Handle<Image>; 4],
    pub surface_dirt: Handle<Image>,
    pub air_tunnel: Handle<Image>,
}

pub fn setup_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let dirt_v = generate_variants(|_, _| [53, 36, 17, 255], 4);
    let stone_v = generate_variants(|x, y| if (x + y) % 13 < 2 { [56, 56, 58, 255] } else { [48, 48, 50, 255] }, 4);
    let sand_v = generate_variants(|x, y| if (x + y) % 7 < 2 { [120, 105, 65, 255] } else { [115, 100, 60, 255] }, 4);
    let grass_v = generate_variants(|x, y| if (x + y) % 9 < 2 { [55, 118, 32, 255] } else { [62, 126, 40, 255] }, 4);
    let wet_v = generate_variants(|_, _| [22, 16, 8, 255], 4);
    let loose_v = generate_variants(|x, y| if (x * 3 + y) % 8 < 2 { [80, 60, 34, 255] } else { [72, 52, 26, 255] }, 4);
    let surface = build_16x16(&build_pixels(|x, y| if x % 3 == 0 && y % 4 == 0 { [55, 118, 32, 255] } else { [62, 126, 40, 255] }));
    let air_t = build_16x16(&build_pixels(|_, _| [18, 13, 9, 255]));

    commands.insert_resource(TextureAssets {
        dirt: arr4(&mut images, dirt_v),
        stone: arr4(&mut images, stone_v),
        sand: arr4(&mut images, sand_v),
        grass: arr4(&mut images, grass_v),
        wet_dirt: arr4(&mut images, wet_v),
        loose_dirt: arr4(&mut images, loose_v),
        surface_dirt: images.add(surface),
        air_tunnel: images.add(air_t),
    });
}

fn arr4(images: &mut Assets<Image>, v: Vec<Image>) -> [Handle<Image>; 4] {
    [images.add(v[0].clone()), images.add(v[1].clone()), images.add(v[2].clone()), images.add(v[3].clone())]
}

pub fn texture_for_material(assets: &TextureAssets, material: ant_simulation::grid::Material, x: u16, y: u16) -> Option<Handle<Image>> {
    let v = hash_pos(x, y) as usize % 4;
    match material {
        ant_simulation::grid::Material::Dirt => Some(assets.dirt[v].clone()),
        ant_simulation::grid::Material::Stone => Some(assets.stone[v].clone()),
        ant_simulation::grid::Material::Sand => Some(assets.sand[v].clone()),
        ant_simulation::grid::Material::LooseDirt => Some(assets.loose_dirt[v].clone()),
        ant_simulation::grid::Material::WetDirt => Some(assets.wet_dirt[v].clone()),
        _ => None,
    }
}

pub fn grass_variant(assets: &TextureAssets, x: u16, _y: u16) -> Handle<Image> {
    assets.grass[hash_pos(x, 0) as usize % 4].clone()
}

pub fn tunnel_texture(assets: &TextureAssets) -> Handle<Image> {
    assets.air_tunnel.clone()
}

fn build_pixels(f: impl Fn(u32, u32) -> [u8; 4]) -> [[u8; 4]; 256] {
    let mut p = [[0u8; 4]; 256];
    for y in 0..16u32 {
        for x in 0..16u32 {
            p[(y * 16 + x) as usize] = f(x, y);
        }
    }
    p
}

fn build_16x16(pixels: &[[u8; 4]; 256]) -> Image {
    let mut data = Vec::with_capacity(1024);
    for p in pixels { data.extend_from_slice(p); }
    Image::new(Extent3d { width: 16, height: 16, depth_or_array_layers: 1 }, TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD)
}

fn generate_variants(f: impl Fn(u32, u32) -> [u8; 4], count: usize) -> Vec<Image> {
    (0..count).map(|v| {
        let mut pixels = [[0u8; 4]; 256];
        let mut rng = simple_rng((v as u32) * 1000);
        for y in 0..16u32 {
            for x in 0..16u32 {
                let base = f(x, y);
                let n = (rng.next() % 12) as i8 - 6;
                pixels[(y * 16 + x) as usize] = [
                    base[0].saturating_add_signed(n),
                    base[1].saturating_add_signed(n),
                    base[2].saturating_add_signed(n / 2),
                    255
                ];
            }
        }
        build_16x16(&pixels)
    }).collect()
}

fn hash_pos(x: u16, y: u16) -> u32 {
    (x as u32).wrapping_mul(374761393).wrapping_add((y as u32).wrapping_mul(668265263))
}

struct SimpleRng(u32);
fn simple_rng(seed: u32) -> SimpleRng { SimpleRng(seed) }
impl SimpleRng {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1103515245).wrapping_add(12345);
        self.0 >> 16
    }
}
