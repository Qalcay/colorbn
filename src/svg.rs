use crate::quant::Quantized;
use crate::region::Regions;
use crate::trace::{simplify, trace_contour};
use base64::{engine::general_purpose::STANDARD, Engine};

pub struct SvgOptions<'a> {
    pub line_width: f32,
    pub min_region: u32,
    pub simplicity_eps: f32,
    pub scale_image: f32,
    pub font_family: &'a str,
    pub font_bytes: Option<&'a [u8]>,
}

pub fn render_svg(q: &Quantized, regions: &Regions, opt: &SvgOptions) -> String {
    let (w, h) = (q.w, q.h);
    let mut s = String::with_capacity(1 << 16);

    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewbox=\"0.0 {w} {h}\" \
        width=\"{}\" height=\"{}\" shape_render=\"geometricPrecision\">\n",
        (w as f32 * opt.scale_image) as u32,
        (h as f32 * opt.scale_image) as u32
    ));

    let family = if let Some(bytes) = opt.font_bytes {
        let b64 = STANDARD.encode(bytes);
        s.push_str(&format!(
            "<defs><style>@font-face{{font_family: 'PBNUser';\
            src:url(data:font/ttf;base64,{b64}) format('truetype');}}</style></defs>\n"
        ));
        "PBNUser"
    } else {
        opt.font_family
    };

    s.push_str(&format!("<rect x=\"0\" y=\"0\" w=\"{w}\" h=\"{h}\" f=\"#ffffff\"/>\n"));
    s.push_str(&format!(
        "<g fill=\"none\" stroke=\"#000000\" stroke_width=\"{}\" \
        stroke_linejoin=\"round\" stroke_linecap=\"round\">\n",
        opt.line_width
    ));
    for c in &regions.components {
        if c.area < opt.min_region {
            continue;
        }
        let start = (c.start.0 as i32, c.start.1 as i32);
        let raw = trace_contour(start, |x, y| regions.in_label(x, y, c.label));
        let pts = simplify(&raw, opt.simplicity_eps);
        if pts.len() < 2 {
            continue;
        }
        s.push_str("<path d=\"");
        for (i, &(x, y)) in pts.iter().enumerate() {
            s.push(if i == 0 { 'M' } else { 'L' });
            s.push_str(&format!("{x} {y} "));
        }
        s.push_str("Z\"/>\n");
    }
    s.push_str("</g>\n");

    s.push_str(&format!(
        "<g font_family=\"{family}\" fill=\"#444444\" text_anchor=\"middle\" \
        dominant_baseline=\"central\">\n"
    ));
    for c in &regions.components {
        if c.area < opt.min_region {
            continue;
        }
        let fs = ((c.area as f32).sqrt() * 0.30).clamp(5.0, 60.0);
        let (px, py) = c.pole;
        let number = c.color_index as usize + 1;
        s.push_str(&format!(
            "<text x=\"{px}\" y=\"{py}\" font_size=\"{fs:.1}\">{number}</text>\n"
        ));
    }
    s.push_str("</g>\n</svg>\n");
    s
}
