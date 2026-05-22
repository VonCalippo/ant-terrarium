use bevy::{prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, render::render_asset::RenderAssetUsages};
use ant_simulation::grid::Direction;

#[derive(Resource)]
pub struct PixelAssets {
    pub ant_right_1: Handle<Image>, pub ant_right_2: Handle<Image>,
    pub ant_right_3: Handle<Image>, pub ant_right_4: Handle<Image>,
    pub ant_left_1: Handle<Image>, pub ant_left_2: Handle<Image>,
    pub ant_left_3: Handle<Image>, pub ant_left_4: Handle<Image>,
    pub ant_up_1: Handle<Image>, pub ant_up_2: Handle<Image>,
    pub ant_up_3: Handle<Image>, pub ant_up_4: Handle<Image>,
    pub ant_down_1: Handle<Image>, pub ant_down_2: Handle<Image>,
    pub ant_down_3: Handle<Image>, pub ant_down_4: Handle<Image>,
    pub queen_sprite: Handle<Image>,
    pub egg_sprite: Handle<Image>,
    pub larva_sprite: Handle<Image>,
    pub fungus_sprite: Handle<Image>,
    pub sky_bg: Handle<Image>,
}

fn make_8x8(pixels: &[u8]) -> Image {
    let mut data = vec![0u8; 256];
    let n = pixels.len().min(256);
    data[..n].copy_from_slice(&pixels[..n]);
    Image::new(
        Extent3d { width: 8, height: 8, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

pub fn setup_pixel_art(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(PixelAssets {
        ant_right_1: images.add(make_8x8(&build_ant_right_1())), 
        ant_right_2: images.add(make_8x8(&build_ant_right_2())),
        ant_right_3: images.add(make_8x8(&build_ant_right_3())), 
        ant_right_4: images.add(make_8x8(&build_ant_right_4())),
        ant_left_1: images.add(make_8x8(&build_ant_left_1())), 
        ant_left_2: images.add(make_8x8(&build_ant_left_2())),
        ant_left_3: images.add(make_8x8(&build_ant_left_3())), 
        ant_left_4: images.add(make_8x8(&build_ant_left_4())),
        ant_up_1: images.add(make_8x8(&build_ant_up_1())), 
        ant_up_2: images.add(make_8x8(&build_ant_up_2())),
        ant_up_3: images.add(make_8x8(&build_ant_up_3())), 
        ant_up_4: images.add(make_8x8(&build_ant_up_4())),
        ant_down_1: images.add(make_8x8(&build_ant_down_1())), 
        ant_down_2: images.add(make_8x8(&build_ant_down_2())),
        ant_down_3: images.add(make_8x8(&build_ant_down_3())), 
        ant_down_4: images.add(make_8x8(&build_ant_down_4())),
        queen_sprite: images.add(make_8x8(&build_queen())),
        egg_sprite: images.add(make_8x8(&build_egg())),
        larva_sprite: images.add(make_8x8(&build_larva())),
        fungus_sprite: images.add(make_8x8(&build_fungus())),
        sky_bg: images.add(sky_image()),
    });
}

pub fn ant_sprite_handle(assets: &PixelAssets, dir: Direction, tick: u64) -> Handle<Image> {
    // 4 frames per direction, cycling through
    let f = ((tick / 4) % 4) as usize;
    match dir {
        Direction::N | Direction::NE | Direction::NW => {
            [&assets.ant_up_1, &assets.ant_up_2, &assets.ant_up_3, &assets.ant_up_4][f].clone()
        }
        Direction::S | Direction::SE | Direction::SW => {
            [&assets.ant_down_1, &assets.ant_down_2, &assets.ant_down_3, &assets.ant_down_4][f].clone()
        }
        Direction::E => {
            [&assets.ant_right_1, &assets.ant_right_2, &assets.ant_right_3, &assets.ant_right_4][f].clone()
        }
        Direction::W => {
            [&assets.ant_left_1, &assets.ant_left_2, &assets.ant_left_3, &assets.ant_left_4][f].clone()
        }
    }
}

// Color helpers
const Z: [u8; 4] = [0,0,0,0];          // transparent
const BD: [u8; 4] = [18,10,6,255];      // body dark
const BM: [u8; 4] = [42,25,15,255];     // body mid
const BL: [u8; 4] = [70,45,28,255];     // body light
const BE: [u8; 4] = [190,55,25,255];    // eye
const GD: [u8; 4] = [210,170,35,255];   // gold
const GC: [u8; 4] = [250,215,75,255];   // crown gold
const QD: [u8; 4] = [35,20,8,255];      // queen dark
const QM: [u8; 4] = [75,45,22,255];     // queen mid
const EW: [u8; 4] = [238,238,222,255];  // egg white
const ES: [u8; 4] = [208,208,192,255];  // egg shadow
const LW: [u8; 4] = [252,238,198,255];  // larva white
const LP: [u8; 4] = [252,198,148,255];  // larva pink
const LE: [u8; 4] = [95,38,38,255];     // larva eye
const FD: [u8; 4] = [38,76,33,255];     // fungus dark
const FG: [u8; 4] = [56,115,47,255];    // fungus green
const FL: [u8; 4] = [95,162,75,255];    // fungus light

fn pix(c: [u8; 4]) -> [u8; 4] { c }

fn emit(target: &mut Vec<u8>, grid: &[[u8; 4]; 64]) {
    for p in grid { target.extend_from_slice(p); }
}

// Walk frames: frame 1 = legs spread, frame 2 = legs together
fn build_ant_right_1() -> Vec<u8> { build_ant_right() }  // original = frame 1
fn build_ant_right_2() -> Vec<u8> { // legs closer together
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,m,m,d,t,t, t,t,d,m,m,e,t,t, t,d,m,m,m,d,d,t,
        d,m,m,m,l,m,d,t, t,d,d,d,d,d,t,t, t,t,d,t,t,d,t,t, t,t,t,t,t,d,t,t,
    ]);
    v
}
fn build_ant_left_1() -> Vec<u8> { build_ant_left() }
fn build_ant_left_2() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,d,d,t,t,t,t, t,t,d,m,m,d,t,t, t,t,e,m,m,d,t,t, t,d,d,m,m,m,d,t,
        t,d,m,l,m,m,m,d, t,t,d,d,d,d,d,t, t,t,d,t,t,d,t,t, t,t,t,t,t,d,t,t,
    ]);
    v
}
fn build_ant_up_1() -> Vec<u8> { build_ant_up() }
fn build_ant_up_2() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, t,t,t,d,d,t,t,t, t,t,t,t,t,d,t,t,
    ]);
    v
}
fn build_ant_down_1() -> Vec<u8> { build_ant_down() }
fn build_ant_down_2() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, t,d,t,d,d,t,d,t, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_right() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,m,m,d,t,t, t,t,d,m,m,e,t,t, t,d,m,m,m,d,d,t,
        d,m,m,m,l,m,d,t, t,d,d,d,d,d,t,t, t,t,d,t,t,d,t,t, t,d,t,t,t,t,d,t,
    ]);
    v
}

fn build_ant_left() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,d,d,t,t,t,t, t,t,d,m,m,d,t,t, t,t,e,m,m,d,t,t, t,d,d,m,m,m,d,t,
        t,d,m,l,m,m,m,d, t,t,d,d,d,d,d,t, t,t,d,t,t,d,t,t, t,d,t,t,t,t,d,t,
    ]);
    v
}

fn build_ant_up() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, t,t,t,d,d,t,t,t, t,t,t,d,d,t,t,t,
    ]);
    v
}

fn build_ant_down() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, t,d,t,d,d,t,d,t, d,t,t,t,t,t,t,d,
    ]);
    v
}

// Additional animation frames for smoother walking
fn build_ant_right_3() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,m,m,d,t,t, t,t,d,m,m,e,t,t, t,d,m,m,m,d,d,t,
        d,m,m,m,l,m,d,t, t,d,d,d,d,d,t,t, d,t,t,d,t,t,t,t, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_right_4() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,m,m,d,t,t, t,t,d,m,m,e,t,t, t,d,m,m,m,d,d,t,
        d,m,m,m,l,m,d,t, t,d,d,d,d,d,t,t, t,d,d,t,t,d,t,t, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_left_3() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,d,d,t,t,t,t, t,t,d,m,m,d,t,t, t,t,e,m,m,d,t,t, t,d,d,m,m,m,d,t,
        t,d,m,l,m,m,m,d, t,t,d,d,d,d,d,t, t,t,t,d,t,t,d,t, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_left_4() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,d,d,t,t,t,t, t,t,d,m,m,d,t,t, t,t,e,m,m,d,t,t, t,d,d,m,m,m,d,t,
        t,d,m,l,m,m,m,d, t,t,d,d,d,d,d,t, t,t,d,d,t,t,d,t, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_up_3() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, d,t,t,d,d,t,t,d, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_up_4() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, t,t,t,d,d,t,t,t, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_down_3() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, d,t,t,d,d,t,t,d, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_ant_down_4() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(BD); let m = pix(BM); let l = pix(BL); let e = pix(BE);
    emit(&mut v, &[
        t,t,t,d,d,t,t,t, t,t,d,d,d,d,t,t, t,d,l,m,m,l,d,t, d,l,m,e,e,m,l,d,
        t,d,m,m,m,m,d,t, t,t,d,m,m,d,t,t, t,d,t,d,d,t,d,t, t,t,t,t,d,t,t,t,
    ]);
    v
}

fn build_queen() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let c = pix(GC); let g = pix(GD); let qd = pix(QD); let qm = pix(QM);
    emit(&mut v, &[
        t,t,c,c,c,c,t,t, t,c,g,g,g,g,c,t, t,t,qd,qm,qm,qd,t,t, qd,qm,qm,qm,qm,qm,qm,qd,
        t,qd,g,qm,qm,g,qd,t, t,qd,t,t,t,t,qd,t, t,qd,t,t,t,t,qd,t, t,t,t,t,t,t,t,t,
    ]);
    v
}

fn build_egg() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let w = pix(EW); let s = pix(ES);
    emit(&mut v, &[
        t,t,w,w,w,w,t,t, t,w,w,w,w,w,w,t, w,w,w,s,w,w,w,w, w,w,w,w,w,w,w,w,
        t,w,w,w,w,w,w,t, t,t,w,w,w,w,t,t, t,t,t,w,w,t,t,t, t,t,t,t,t,t,t,t,
    ]);
    v
}

fn build_larva() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let w = pix(LW); let p = pix(LP); let e = pix(LE);
    emit(&mut v, &[
        t,t,t,w,w,t,t,t, t,t,w,w,w,w,t,t, t,w,w,p,p,w,w,t, w,w,p,w,w,p,w,w,
        w,w,w,e,e,w,w,w, t,w,w,w,w,w,w,t, t,t,w,w,w,w,t,t, t,t,t,w,w,t,t,t,
    ]);
    v
}

fn build_fungus() -> Vec<u8> {
    let mut v = Vec::with_capacity(256);
    let t = pix(Z); let d = pix(FD); let g = pix(FG); let l = pix(FL);
    emit(&mut v, &[
        t,t,d,d,d,d,t,t, t,d,g,g,g,g,d,t, d,g,l,g,g,l,g,d, d,g,g,g,g,g,g,d,
        t,d,g,g,g,g,d,t, t,t,d,g,g,d,t,t, t,t,t,d,d,t,t,t, t,t,t,d,d,t,t,t,
    ]);
    v
}

fn sky_image() -> Image {
    let mut data = Vec::with_capacity(72 * 4);
    for y in 0u8..72 {
        let t = y as f32 / 71.0;
        let r = 15 + (t * 20.0) as u8;
        let g = 20 + (t * 25.0) as u8;
        let b = 40 + (t * 35.0) as u8;
        data.extend_from_slice(&[r, g, b, 255]);
    }
    Image::new(
        Extent3d { width: 1, height: 72, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
