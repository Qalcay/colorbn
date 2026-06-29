const N8: [(i32, i32); 8] = [
    (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1),
];

fn n8_pos(d: (i32, i32)) -> usize {
    N8.iter().position(|&o| o == d).expect("offset must be an 8-neighbour!")
}

pub fn trace_contour<F: Fn(i32, i32) -> bool>(start: (i32, i32), fg: F) -> Vec<(i32, i32)> {
    let (mut cx, mut cy) = start;
    let (mut bx, mut by) = (cx - 1, cy);
    let mut contour = vec![(cx, cy)];
    let mut first_next: Option<(i32, i32)> = None;

    let cap = 1_000_000u32;
    let mut guard = 0u32;



    loop {
        guard += 1;
        if guard > cap {
            return bounding_box_fallback(&contour);
        }

        let bi = n8_pos((bx - cx, by - cy));
        let mut prev = (bx, by);
        let mut found: Option<(i32, i32)> = None;
        for s in 1..=8 {
            let idx = (bi + s) % 8;
            let (nx, ny) = (cx + N8[idx].0, cy + N8[idx].1);
            if fg(nx, ny) {
                found = Some((nx, ny));
                break;
            }
            prev = (nx, ny);
        }
        let (nx, ny) = match found {
            Some(p) => p,
            None => return bounding_box_fallback(&contour),
        };

        match first_next {
            None => first_next = Some((nx, ny)),
            Some(fn0) => {
                if (cx, cy) == start && (nx, ny) == fn0 {
                    break;
                }
            }
        }

        bx = prev.0;
        by = prev.1;
        cx = nx;
        cy = ny;
        contour.push((cx, cy));
    }
    contour
}

fn bounding_box_fallback(pts: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for &(x, y) in pts {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)]
}

pub fn simplify(points: &[(i32, i32)], eps: f32) -> Vec<(i32, i32)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut out = Vec::new();
    dp(points, eps, &mut out);
    out.push(*points.last().unwrap());
    out
}

pub fn dp(points: &[(i32, i32)], eps: f32, out: &mut Vec<(i32, i32)>) {
    let (a, b) = (points[0], points[points.len() - 1]);
    let mut dmax = 0.0f32;
    let mut idx = 0usize;
    for i in 1..points.len() - 1 {
        let d = perp_dist(points[i], a, b);
        if d > dmax {
            dmax = d;
            idx = i;
        }
    }
    if dmax > eps {
        dp(&points[..=idx], eps, out);
        dp(&points[idx..], eps, out);
    } else {
        out.push(a);
    }
}

fn perp_dist(p: (i32, i32), a: (i32, i32), b: (i32, i32)) -> f32 {
    let (px, py) = (p.0 as f32, p.1 as f32);
    let (ax, ay) = (a.0 as f32, a.1 as f32);
    let (bx, by) = (b.0 as f32, b.1 as f32);
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / len2;
    let (qx, qy) = (ax + t * dx, ay + t * dy);
    ((px - qx).powi(2) + (py - qy).powi(2)).sqrt()
}
