use core::num;
use js_sys::{ArrayBuffer, Uint8Array};

use write_fonts::read::{
    FontRef, 
    TableProvider
};
use write_fonts::{
    from_obj::ToOwnedTable,
    tables::os2::Os2,
    tables::maxp::Maxp,
    FontBuilder,
};
use woff2::decode::{convert_woff2_to_ttf, is_woff2};
use font_types::GlyphId;


fn get_font_path() -> String {
    let mut input_path_str = if let Ok(path) = std::env::current_dir() {
        print!("The current directory is {}\n", path.display());
        path.display().to_string()
    } else {
        print!("Can't access current directory");
        "".to_string()
    };

    input_path_str += "/test/fonts/NotoSans-Regular.ttf";
    input_path_str
}

fn print_glyphs_reg(font: &FontRef) -> String {
    let num_glyphs: u16 = font.maxp().unwrap().num_glyphs();
    let loca = font.loca(None).unwrap();
    let glyf = font.glyf().unwrap();
    for gid in 0..num_glyphs {
        println!("Glyph: {}", gid);
        let glyph = loca.get_glyf(GlyphId::new(gid), &glyf).unwrap().unwrap();

        println!("  DATA: {:#?}", glyph);
    }

    return format!("Done");
}

fn print_glyphs(rust_buf: Vec<u8>, num_glyphs: u16) -> String {
    use skrifa::{
        instance::LocationRef, outline::DrawSettings, prelude::Size, raw::FontRef, scale::Pen,
        MetadataProvider,
    };

    #[derive(Default)]
    struct SvgPen {
        min_x: Option<f32>,
        min_y: Option<f32>,
        max_x: Option<f32>,
        max_y: Option<f32>,
        fragments: Vec<String>,
    }
    
    fn min(f1: f32, f2: f32) -> f32 {
        if f1 < f2 {
            f1
        } else {
            f2
        }
    }
    
    fn max(f1: f32, f2: f32) -> f32 {
        if f1 > f2 {
            f1
        } else {
            f2
        }
    }
    
    fn update_extent(opt: &mut Option<f32>, v: f32, cmp: fn(f32, f32) -> f32) {
        *opt = Some(match opt {
            Some(v2) => cmp(v, *v2),
            None => v,
        });
    }
    
    impl SvgPen {
        fn update_extents(&mut self, x: f32, y: f32) {
            update_extent(&mut self.min_x, x, min);
            update_extent(&mut self.min_y, y, min);
            update_extent(&mut self.max_x, x, max);
            update_extent(&mut self.max_y, y, max);
        }
    
        fn to_string(mut self) -> String {
            let min_x = self.min_x.unwrap_or_default();
            let min_y = self.min_y.unwrap_or_default();
            let max_y = self.max_y.unwrap_or_default();
            let width = self.max_x.unwrap_or_default() - min_x;
            let height = max_y - min_y;
    
            // To flip over y at the middle of the shape we would translate down so the middle
            // is at 0, flip y, then translate back up again. The translates add up so we end up
            // shifting by twice the midpoint.
            let shift = min_y + max_y;
    
            self.fragments.insert(0, format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x} {min_y} {width} {height}">"#));
            self.fragments
                .insert(1, format!(r#"<g transform="matrix(1 0 0 -1 0 {shift})">"#));
            self.fragments.insert(2, r#"<path d=""#.to_string());
            self.fragments.push(r#""/>"#.to_string());
            self.fragments.push("</g>".to_string());
            self.fragments.push("</svg>".to_string());
            self.fragments.join(" ")
        }
    }
    
    impl Pen for SvgPen {
        fn move_to(&mut self, x: f32, y: f32) {
            self.fragments.push(format!("M{x:.3},{y:.3}"));
            self.update_extents(x, y);
        }
    
        fn line_to(&mut self, x: f32, y: f32) {
            self.fragments.push(format!("L{x:.3},{y:.3}"));
            self.update_extents(x, y);
        }
    
        fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
            self.fragments
                .push(format!("Q{cx0:.3},{cy0:.3} {x:.3},{y:.3}"));
            self.update_extents(cx0, cy0);
            self.update_extents(x, y);
        }
    
        fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
            self.fragments.push(format!(
                "C{cx0:.3},{cy0:.3} {cx1:.3},{cy1:.3} {x:.3},{y:.3}"
            ));
            self.update_extents(cx0, cy0);
            self.update_extents(cx1, cy1);
            self.update_extents(x, y);
        }
    
        fn close(&mut self) {
            self.fragments.push("z".to_string());
        }
    }

    

    let font = match FontRef::new(&rust_buf) {
        Ok(font) => font,
        Err(e) => return format!("FontRef::new failed: {e}"),
    };

    
    // let var_location = font.axes().location(&[("wght", 650.0), ("wdth", 100.0)]);
    // let settings = DrawSettings::unhinted(Size::new(16.0), &var_location);
    for gid in 0..num_glyphs {
        println!("GlyphID: {}", gid);
        let glyph = match font.outline_glyphs().get(GlyphId::new(gid)) {
            Some(glyph) => glyph,
            None => return format!("No outline for glyph"),
        };
        
        let settings = DrawSettings::unhinted(Size::new(16.0), LocationRef::default());
        let mut pen = SvgPen::default();

        let glyph_svg = match glyph.draw(settings, &mut pen) {
            Ok(..) => pen.to_string(),
            Err(e) => format!("outline failed: {e}"),
        };
        println!("Glyph SVG: {:?}", glyph_svg);

    }


    return format!("Done");
}


fn main () {
    //let input_path_str = "/Users/ollimeier/Documents/hithub_data/ollimeier/demo-rust/NotoSans-Regular.ttf";
    let input_path_str = &get_font_path();

    let path_to_my_font_file = std::path::Path::new(input_path_str);
    let font_bytes = std::fs::read(path_to_my_font_file).unwrap();
    let font = FontRef::new(&font_bytes).expect("failed to read font data");
    let mut os2: Os2 = font.os2().expect("missing 'os/2' table").to_owned_table();
    os2.us_weight_class = 444;

    
    //print_glyphs_reg(&font);
    let num_glyphs: u16 = font.maxp().unwrap().num_glyphs();
    print_glyphs(font_bytes.clone(), num_glyphs);
    println!("Number glyphs {}", num_glyphs);


    let new_bytes = FontBuilder::new()
        .add_table(&os2)
        .unwrap() // errors if we can't compile 'head', unlikely here
        .copy_missing_tables(font)
        .build();
 
    std::fs::write(input_path_str.replace(".ttf", "-mod.ttf"), &new_bytes).unwrap();
}
