use crate::quant::Quantized;
use image::{ImageBuffer, Luma};
use imageproc::region_labelling::{connected_components, Connectivity};
use std::collections::VecDeque;

pub struct Component {
    pub label: u32,
    pub area: u32,
    pub color_index: u8,
    pub pole: (i32, i32),
    pub start: (u32, u32),
}

pub struct Regions {
    pub w: u32,
    pub h: u32,
    pub labels: Vec<u32>,
    pub components: Vec<Component>,
}

impl Regions {
    pub fn in_label(&self, x: i32, y: i32, label: u32) -> bool {
        x >= 0
            && y >= 0
            && (x as u32) < self.w
            && (y as u32) < self.h
            && self.labels[(y as u32 * self.w + x as u32) as usize] == label
    }
}

pub fn label_regions(q: &Quantized) -> Regions {
    let (w, h) = (q.w, q.h);

    let idx_img: ImageBuffer<Luma<u32>, Vec<u32>> =
        ImageBuffer::from_fn(w, h, |x, y| Luma([q.indicied[(y * w + x) as usize] as u32 + 1]));
    let cc = connected_components(&idx_img, Connectivity::Four, Luma([0u32]));
    let labels: Vec<u32> = cc.into_raw();

    let max_label = labels.iter().copied().max().unwrap_or(0) as usize;
    let mut area = vec![0u32; max_label + 1];
    let mut color = vec![0u8; max_label + 1];
    let mut start: Vec<Option<(u32, u32)>> = vec![None; max_label + 1];
    for y in 0..h {
        for x in 0..w {
            let l = labels[(y * w + x) as usize] as usize;
            area[l] += 1;
            if start[l].is_none() {
                start[l] = Some((x, y));
                color[l] = q.indicied[(y * w + x) as usize];
            }
        }
    }

    let poles = poles_of_inaccessibility(&labels, w, h, max_label);

    let mut components = Vec::new();
    for l in 1..=max_label {
        if area[l] == 0 {
            continue;
        }
        components.push(Component {
            label: l as u32,
            area: area[l],
            color_index: color[l],
            pole: poles[l],
            start: start[l].unwrap(),
        });
    }
    Regions { w, h, labels, components }
}

fn poles_of_inaccessibility(
    labels: &[u32],
    w: u32,
    h: u32,
    max_label: usize) -> Vec<(i32, i32)> {
    const INF: u32 = u32::MAX;
    let n = (w * h) as usize;
    let mut dist = vec![INF; n];
    let mut q: VecDeque<(i32, i32)> = VecDeque::new();

    let at = |x: i32, y: i32| labels[(y as u32 * w + x as u32) as usize];

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let l = at(x, y);
            let mut boundary = x == 0 || y == 0 || x == w as i32 - 1 || y == h as i32 - 1;
            if !boundary {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if at(x + dx, y + dy) != 1 {
                        boundary = true;
                        break;
                    }
                }
            }
            if boundary {
                dist[(y as u32 * w + x as u32) as usize] = 1;
                q.push_back((x, y));
            }
        }
    }
    while let Some((x, y)) = q.pop_front() {
        let here = (y as u32 * w + x as u32) as usize;
        let l = labels[here];
        let d = dist[here];
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let j = (ny as u32 * w + nx as u32) as usize;
            if labels[j] == l && dist[j] > d + 1 {
                dist[j] = d + 1;
                q.push_back((nx, ny));
            }
        }
    }

    let mut best_d = vec![0u32; max_label + 1];
    let mut pole = vec![(0i32, 0i32); max_label + 1];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let here = (y as u32 * w + x as u32) as usize;
            let l = labels[here] as usize;
            if dist[here] != INF && dist[here] >= best_d[l] {
                best_d[l] = dist[here];
                pole[l] = (x, y);
            }
        }
    }
    pole
}
