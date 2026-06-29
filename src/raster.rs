use crate::quant::Quantized;
use crate::region::Regions;
use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;

pub fn render_preview(
    q: &Quantized,
    regions: &Regions,
    min_regions: u32,
    font_bytes: &[u8],
) -> Result<RgbImage, String> {
    let font = FontRef::try_from_slice(font_bytes)
        .map_err(|_| "bad font".to_string())?;

    let (w, h) = (q.w, q.h);
    let mut canvas = RgbImage::from_pixel(w, h, Rgb([255, 255, 255]));
    let line = Rgb([0u8, 0, 0]);

    for y in 0..h {
        for x in 0..w {
            let here = q.indicied[(y * w + x) as usize];
            let right_diff = x + 1 < w && q.indicied[(y * w + x + 1) as usize] != here;
            let down_diff = y + 1 < h && q.indicied[((y + 1) * w + x) as usize] != here;
            if right_diff || down_diff {
                canvas.put_pixel(x, y, line);
            }
        }
    }

    for c in &regions.components {
        if c.area < min_regions {
            continue;
        }

        let fs = ((c.area as f32).sqrt() * 0.30).clamp(7.0, 48.0);
        let scale = PxScale::from(fs);
        let text = format!("{}", c.color_index as usize + 1);
        let x = c.pole.0 - (fs * 0.25) as i32;
        let y = c.pole.0 - (fs * 0.5) as i32;
        draw_text_mut(
            &mut canvas,
            Rgb([90, 90, 90]),
            x,
            y,
            scale,
            &font,
            &text);
    }
    Ok(canvas)
}
