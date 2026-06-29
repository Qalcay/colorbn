use image::{imageops::FilterType, GenericImageView, RgbImage};
use kmeans_colors::get_kmeans;
use palette::{FromColor, Lab, Srgb};

#[derive(Clone, Debug)]
pub struct Swatch {
    pub num: usize,
    pub rgb: [u8; 3],
    pub fra: f32,
}

pub struct Quantized {
    pub w: u32,
    pub h: u32,
    pub indicied: Vec<u8>,
    pub palette: Vec<Swatch>,
    pub score: f32,
}

pub fn load_and_resize(path: &str, max_dim: u32) -> Result<RgbImage, String> {
    let img = image::open(path).map_err(|e| format!(" {path}: {e}"))?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Err("".into());
    }
    let (nw, nh) = if w >= h {
        (max_dim, ((max_dim as f32) * (h as f32 / w as f32)).round().max(1.0) as u32)
    } else {
        (((max_dim as f32) * (w as f32 / h as f32)).round().max(1.0) as u32, max_dim)
    };
    Ok(img.resize_exact(nw, nh, FilterType::Lanczos3).to_rgb8())
}

pub fn pasteurize(
    img: &RgbImage,
    k: usize,
    variants: usize,
    max_iter: usize,
    converge: f32) -> Quantized {
        let (w, h) = (img.width(), img.height());

        let lab: Vec<Lab> = img
            .pixels()
            .map(|p| {
                let s = Srgb::new(
                    p[0] as f32 / 255.0,
                    p[1] as f32 / 255.0,
                    p[2] as f32 / 255.0
                );
                Lab::from_color(s)
            }).collect();

        let mut best = get_kmeans(k, max_iter, converge, false, &lab, 0);
        println!(" seed 0: score {:.1}", best.score);
        for seed in 1..variants as u64 {
            let run = get_kmeans(k, max_iter, converge, false, &lab, seed);
            println!(" seed {seed}: score {:.1}", run.score);
            if run.score < best.score {
                best = run;
            }
        }
        println!(" -> keeping score {:.1}", best.score);

        let counts = count_indices(&best.indices, k);
        let mut order: Vec<usize> = (0..k).collect();
        order.sort_by(|&a, &b| counts[b].cmp(&counts[a]));
        let mut remap = vec![0u8; k];
        for (new_pos, &old) in order.iter().enumerate() {
            remap[old] = new_pos as u8;
        }
        let total = best.indices.len().max(1) as f32;
        let mut palette = vec![
            Swatch { num: 0, rgb: [0, 0, 0], fra: 0.0 };
            k
        ];
        for (new_pos, &old) in order.iter().enumerate() {
            palette[new_pos] = Swatch {
                num: new_pos + 1,
                rgb: lab_to_rgb(best.centroids[old]),
                fra: counts[old] as f32 / total,
            };
        }
        let indicied: Vec<u8> = best.indices.iter().map(|&c| remap[c as usize]).collect();
        Quantized { w, h, indicied, palette, score: best.score }
}

fn count_indices(indices: &[u8], k: usize) -> Vec<usize> {
    let mut c = vec![0usize; k];
    for &i in indices {
        c[i as usize] += 1;
    }
    c
}

fn lab_to_rgb(lab: Lab) -> [u8; 3] {
    let s = Srgb::from_color(lab);
    [
        (s.red * 255.0).round().clamp(0.0, 255.0) as u8,
        (s.green * 255.0).round().clamp(0.0, 255.0) as u8,
        (s.blue * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}
