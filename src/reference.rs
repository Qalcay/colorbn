use crate::quant::Quantized;

pub const NAMED_PAINT: &[(&str, [u8; 3])] = &[
    ("Bright White",    [245, 245, 239]),
    ("Jet Black",       [27, 27, 29]),
    ("Bold Red",        [198, 30, 54]),
    ("Citrine Yellow",  [234, 189, 9]),
    ("Marine Blue",     [18, 11, 144]),
    ("Petal Green",     [18, 54, 36]),
    ("Golden Brow",     [133, 54, 18]),
    ("Subtle Orange",   [202, 117, 33]),
    ("Dark Purple",     [66, 22, 77]),
    ("Sky Blue",        [45, 81, 189]),
    ("Earthy Green",    [77, 121, 45]),
    ("Subtle Amber",    [111, 75, 18]),
];

pub struct LegendRow {
    pub number: usize,
    pub rgb: [u8; 3],
    pub hex: String,
    pub name: Option<String>,
    pub perc: f32,
}

pub fn build_legend(q: &Quantized, use_names: bool) -> Vec<LegendRow> {
    q.palette.iter().map(
        |sw| {
            let (rgb, name) = if use_names {
                match NAMED_PAINT.get(sw.num - 1) {
                    Some((n, c)) => (*c, Some((*n).to_string())),
                    None => (sw.rgb, None), // too many clusters to named paints -> set back
                }
            } else {
                (sw.rgb, None)
            };
            LegendRow {
                number: sw.num,
                rgb,
                hex: format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
                name,
                perc: sw.fra * 100.0,
            }
        }
    ).collect()
}

pub fn print_legend(rows: &[LegendRow]) {
    println!("   #    swatch    coverage    name");
    for r in rows {
        let name = r.name.as_deref().unwrap_or("");
        println!(" {:>2}   {:<9}   {:>6.1}%    {}", r.number, r.hex, r.perc, name);
    }
}

pub fn render_reference_svg(rows: &[LegendRow], font_family: &str) -> String {
    let row_h = 40u32;
    let pad = 16u32;
    let sw = 28u32; // palette size
    let width = 360u32;
    let height = pad * 2 + row_h * rows.len() as u32;

    let mut s = String::with_capacity(1024 + rows.len() * 160);
    s.push_str(&format!("
        <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0.0 {width} {height}\" \
        width=\"{width}\" height=\"{height}\" font-family=\"{font_family}\">\n"));
    s.push_str(&format!(
        "<rect width=\"{width}\" height=\"{height}\" fill=\"#ffffff\"/>\n"));
    s.push_str(&format!(
        "<text x=\"{pad}\" y=\"{}\" font_size=\"16\" font_weight=\"bold\" fill=\"#111\"> Reference</text>\n",
        pad
    ));

    for (i, r) in rows.iter().enumerate() {
        let y = pad + 8 + row_h * i as u32;
        let hex = &r.hex;
        // number
        s.push_str(&format!(
            "<text
            x=\"{}\"
            y=\"{}\"
            font_size=\"18\"
            fill=\"#111\"
            text_anchor=\"middle\"
            dominant_baseline=\"central\">{}</text>\n",
            pad + 8,
            y + sw / 2,
            r.number
        ));
        // swatch
        s.push_str(&format!(
            "<rect
            x=\"{}\"
            y=\"{}\"
            w=\"{sw}\"
            h=\"{sw}\"
            rx=\"4\"
            fill=\"{hex}\"
            stroke=\"#999\"/>\n",
            pad + 28,
            y
        ));
        // label
        let label = match &r.name {
            Some(n) => format!("{n}  {hex}  ({:.1}%)", r.perc),
            None => format!("{hex}  ({:.1}%)", r.perc),
        };
        s.push_str(&format!(
            "<text
            x=\"{}\"
            y=\"{}\"
            font_size=\"13\"
            fill=\"#333\"
            dominant_baseline=\"central\">{}</text>\n",
            pad + 28 + sw as u32 + 12,
            y + sw / 2,
            xml_escape(&label)
        ));
    }
    s.push_str("</svg>\n");
    s
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
