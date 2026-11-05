//! Decode the original game's uncompressed 24-bit BMP atlases at startup.
//! White color keys are applied in memory; the source artwork remains unchanged.
use macroquad::prelude::*;

pub struct Assets {
    pub grass: Texture2D,
    pub objects: Texture2D,
    pub brown: Texture2D,
    pub green: Texture2D,
}
impl Assets {
    pub fn new() -> Self {
        Self {
            grass: texture(include_bytes!("../Sprites/Tiles.bmp"), false),
            objects: texture(include_bytes!("../Sprites/Obj.bmp"), true),
            brown: texture(include_bytes!("../Sprites/spr1.bmp"), true),
            green: texture(include_bytes!("../Sprites/spr2.bmp"), true),
        }
    }
}
fn texture(bytes: &[u8], key: bool) -> Texture2D {
    let image = decode_bmp(bytes, key).expect("embedded BMP atlas must be valid");
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Linear);
    texture
}
fn decode_bmp(bytes: &[u8], key: bool) -> Result<Image, &'static str> {
    if bytes.len() < 54 || &bytes[..2] != b"BM" {
        return Err("invalid BMP header");
    }
    let u32_at = |i| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    if u16::from_le_bytes(bytes[28..30].try_into().unwrap()) != 24 || u32_at(30) != 0 {
        return Err("BMP must be uncompressed RGB24");
    }
    let (width, height, offset) = (
        u32_at(18) as usize,
        u32_at(22) as usize,
        u32_at(10) as usize,
    );
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err("invalid atlas dimensions");
    }
    let stride = (width * 3).div_ceil(4) * 4;
    if offset + stride * height > bytes.len() {
        return Err("truncated atlas");
    }
    let mut rgba = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let src = offset + (height - 1 - y) * stride + x * 3;
            let dst = (y * width + x) * 4;
            let (b, g, r) = (bytes[src], bytes[src + 1], bytes[src + 2]);
            rgba[dst..dst + 4].copy_from_slice(&[
                r,
                g,
                b,
                if key && r == 255 && g == 255 && b == 255 {
                    0
                } else {
                    255
                },
            ]);
        }
    }
    Ok(Image {
        bytes: rgba,
        width: width as u16,
        height: height as u16,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_monster_atlas_decodes_with_transparency() {
        let image = decode_bmp(include_bytes!("../Sprites/spr1.bmp"), true).unwrap();
        assert_eq!((image.width, image.height), (400, 450));
        assert_eq!(image.bytes[3], 0);
        assert!(image.bytes.as_chunks::<4>().0.iter().any(|p| p[3] == 255));
        assert!(decode_bmp(&[0; 54], true).is_err());
    }
}
