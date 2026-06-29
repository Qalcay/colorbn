






mod quant;
mod raster;
mod reference;
mod region;
mod svg;
mod trace;

use quant::Quantized;
use region::Regions;
use std::io::{self, Write};
use std::path::Path;

const K_MIN: usize = 2;
const K_MAX: usize = 64;
const VAR_MIN: usize = 1;
const VAR_MAX: usize = 50;
const DIM_MIN: u32 = 64;
const DIM_MAX: u32 = 8000;

struct Config {
    k: usize,
    max_dim: u32,
    variance: usize,
    min_regions: u32,
    line_width: f32,
    simplicity_eps: f32,
    scale_image: f32,
    use_names: bool,
    max_iter: usize,
    converge: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            k: 12,
            max_dim: 800,
            variance: 5,
            min_regions: 120,
            line_width: 0.6,
            simplicity_eps: 1.0,
            scale_image: 1.0,
            use_names: false,
            max_iter: 20,
            converge: 1.0,
        }
    }
}

struct Session {
    cfg: Config,
    font_bytes: Option<Vec<u8>>,
    image: Option<image::RgbImage>,
    image_path: Option<String>,
    quant: Option<Quantized>,
    regions: Option<Regions>,
}

fn main() {
    let mut session = Session {
        cfg: Config::default(),
        font_bytes: None,
        image: None,
        image_path: None,
        quant: None,
        regions: None,
    };

    println!("[..[.[ color-by-numbers image gen ].]..]");
    println!("\t 'help' | 'quit' ");

    loop {
        print!("cbn> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if io::stdin().read_line(&mut line)
            .unwrap_or(0) == 0 {
                break; // eof
        }
        let parts: Vec<&str> = line
            .trim()
            .split_whitespace()
            .collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "help" => print_help(),
            "status" => status(&session),
            "load" => cmd_load(&mut session, &parts),
            "set" => cmd_set(&mut session, &parts),
            "pasteurize" | "quant" => cmd_pasteurize(&mut session),
            "colors" | "colours" => cmd_colors(&session),
            "reference" | "ref" => cmd_reference(&session, &parts),
            "svg" => cmd_svg(&session, &parts),
            "preview" => cmd_preview(&session, &parts),
            "quit" | "exit" => break,
            other => println!("'{other}' ('help')"),
        }
    }
}

fn print_help() {
    println!(

    );
}

fn status(s: &Session) {
    let c = &s.cfg;
    println!("settings:");
    println!(" palette={} | max_dim={} | variants={}",
        c.k,
        c.max_dim,
        c.variance);
    println!(" min_regions={} | line={} | eps={} | scale={}",
        c.min_regions,
        c.line_width,
        c.simplicity_eps,
        c.scale_image);
    println!(" names={} | font={}",
        c.use_names,
        if s.font_bytes.is_some() { "set" } else { "none" }
    );
    println!("state: ");
    println!(" image={} | quant={} | regions={}",
        s.image_path.as_deref().unwrap_or("none"),
        if s.quant.is_some() { "yes" } else { "no" },
        s.regions.as_ref().map(|r| r.components.len()).unwrap_or(0)
    );
}

fn cmd_load(s: &mut Session, parts: &[&str]) {
    let Some(path) = parts.get(1) else {
        println!("'load <path>'");
        return;
    };
    match quant::load_and_resize(path, s.cfg.max_dim) {
        Ok(img) => {
            println!("loaded '{}' -> {}x{}",
                path, img.width(), img.height());
            s.image = Some(img);
            s.image_path = Some((*path).to_string());
            s.quant = None; // if settings/image cahnged; force re-pasteuriing
            s.regions = None;
        }
        Err(e) => println!("err: {e}"),
    }
}

fn cmd_set(s: &mut Session, parts: &[&str]) {
    let (Some(key), Some(val)) = (parts.get(1), parts.get(2)) else {
        println!("'set <key> <value>'");
        return;
    };
    match *key {
        "k" => {
            if let Some(v) = parse_clamp_usize(val, K_MIN, K_MAX) {
                s.cfg.k = v;
                s.quant = None;
            }
        }
        "max" => {
            if let Some(v) = parse_clamp_u32(val, DIM_MIN, DIM_MAX) {
                s.cfg.max_dim = v;
                // re-load new res from /path/***.img
                if let Some(p) = s.image_path.clone() {
                    cmd_load(s, &["load", p.as_str()]);
                }
            }
        }
        "variants" => {
            if let Some(v) = parse_clamp_usize(val, VAR_MIN, VAR_MAX) {
                s.cfg.variance = v;
                s.quant = None;
            }
        }
        "min" => {
            if let Some(v) = parse_clamp_u32(val, 0, u32::MAX) {
                s.cfg.min_regions = v;
            }
        }
        "line" => {
            if let Some(v) = parse_f32(val) {
                s.cfg.line_width = v.max(0.05);
            }
        }
        "eps" => {
            if let Some(v) = parse_f32(val) {
                s.cfg.simplicity_eps = v.max(0.0);
            }
        }
        "scale" => {
            if let Some(v) = parse_f32(val) {
                s.cfg.scale_image = v.clamp(0.1, 50.0);
            }
        }
        "names" => match *val {
            "on" => s.cfg.use_names = true,
            "off" => s.cfg.use_names = false,
            _ => println!(" names 'on'|'off'"),
        },
        "font" => match std::fs::read(val) {
            Ok(bytes) => {
                println!("font loaded (as {} bytes)", bytes.len());
                s.font_bytes = Some(bytes);
            }
            Err(e) => println!(" unavailable '{val}': {e}"),
        },
        other => println!(" unknown '{other}'?"),
    }
}

fn cmd_pasteurize(s: &mut Session) {
    let Some(img) = &s.image else {
        println!("load an image please!");
        return;
    };
    println!(
        "pasteurize: k={} | variance={} ({}x{})",
        s.cfg.k,
        s.cfg.variance,
        img.width(),
        img.height()
    );
    let q = quant::pasteurize(
        img,
        s.cfg.k,
        s.cfg.variance,
        s.cfg.max_iter,
        s.cfg.converge);
    let r = region::label_regions(&q);
    let kept = r.components.iter().filter(|c| c.area >= s.cfg.min_regions).count();
    println!(
        " {} regions, {} above min_regions={}",
        r.components.len(),
        kept,
        s.cfg.min_regions
    );
    s.quant = Some(q);
    s.regions = Some(r);
}

fn cmd_colors(s: &Session) {
    let Some(q) = &s.quant else {
        println!("run 'pasteurize' first!");
        return;
    };
    let rows = reference::build_legend(q, s.cfg.use_names);
    reference::print_legend(&rows);
}

fn cmd_reference(s: &Session, parts: &[&str]) {
    let Some(q) = &s.quant else {
        println!("run 'pasteurize' first!");
        return;
    };
    let out = parts.get(1).copied().unwrap_or("reference.svg");
    let rows = reference::build_legend(q, s.cfg.use_names);
    reference::print_legend(&rows);
    let family = font_family(s);
    let svg = reference::render_reference_svg(&rows, family);
    write_file(out, svg.as_bytes());
}

fn cmd_svg(s: &Session, parts: &[&str]) {
    let (Some(q), Some(r)) = (&s.quant, &s.regions) else {
        println!("run 'pasteurize' first!");
        return;
    };
    let out = parts.get(1).copied().unwrap_or("default.svg");
    let opt = svg::SvgOptions {
        line_width: s.cfg.line_width,
        min_region: s.cfg.min_regions,
        simplicity_eps: s.cfg.simplicity_eps,
        scale_image: s.cfg.scale_image,
        font_family: font_family(s),
        font_bytes: s.font_bytes.as_deref(),
    };
    let doc = svg::render_svg(q, r, &opt);
    write_file(out, doc.as_bytes());
}

fn cmd_preview(s: &Session, parts: &[&str]) {
    let (Some(q), Some(r)) = (&s.quant, &s.regions) else {
        println!("run 'pasteurize' first!");
        return;
    };
    let Some(font) = &s.font_bytes else {
        println!("'needs a font: 'set font <path.ttf>' (or '.svg' only)");
        return;
    };
    let stem = s
        .image_path
        .as_deref()
        .and_then(|p| Path::new(p).file_stem())
        .and_then(|x| x.to_str())
        .unwrap_or("default");
    let out = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| format!("pre_{stem}.png"));
    match raster::render_preview(q, r, s.cfg.min_regions, font) {
        Ok(canvas) => match canvas.save(&out) {
            Ok(()) => println!("{out}"),
            Err(e) => println!("unable to save ''{out}: {e}''"),
        },
        Err(e) => println!("err: {e}"),
    }
}

// helping functions


fn font_family(s: &Session) -> &'static str {
    if s.font_bytes.is_some() {
        "PBNUser"
    } else {
        "sans-serif"
    }
}

fn write_file(path: &str, bytes: &[u8]) {
    match std::fs::write(path, bytes) {
        Ok(()) => println!(" {path} (as {} bytes)", bytes.len()),
        Err(e) => println!("unable to write ''{path}'' -> {e}"),
    }
}

fn parse_clamp_usize(v: &str, lo: usize, hi: usize) -> Option<usize> {
    match v.parse::<usize>() {
        Ok(n) => {
            let c = n.clamp(lo, hi);
            if c != n {
                println!("note: {n} clamped to {c} (allowed {lo}..={hi})");
            }
            Some(c)
        }
        Err(_) => {
            println!("'{v}' is not a whole number!");
            None
        }
    }
}

fn parse_clamp_u32(v: &str, lo: u32, hi: u32) -> Option<u32> {
    match v.parse::<u32>() {
        Ok(n) => {
            let c = n.clamp(lo, hi);
            if c != n {
                println!("note: {n} clamped to {c} (allowed {lo}..={hi})");
            }
            Some(c)
        }
        Err(_) => {
            println!("'{v}' is not a whole number!");
            None
        }
    }
}

fn parse_f32(v: &str) -> Option<f32> {
    match v.parse::<f32>() {
        Ok(n) => Some(n),
        Err(_) => {
            println!("'{v}' is not a valid number!");
            None
        }
    }
}
