use write_fonts::read::FontRef;
use write_fonts::read::TableProvider;
use write_fonts::FontBuilder;
use write_fonts::{
    from_obj::ToOwnedTable,
    tables::os2::Os2,
};

fn main () {
    let input_path_str = "/Users/ollimeier/Documents/hithub_data/ollimeier/demo-rust/NotoSans-Regular.ttf";
    let path_to_my_font_file = std::path::Path::new(input_path_str);
    let font_bytes = std::fs::read(path_to_my_font_file).unwrap();
    let font = FontRef::new(&font_bytes).expect("failed to read font data");
    let mut os2: Os2 = font.os2().expect("missing 'os/2' table").to_owned_table();
    os2.us_weight_class = 444;

    let new_bytes = FontBuilder::new()
        .add_table(&os2)
        .unwrap() // errors if we can't compile 'head', unlikely here
        .copy_missing_tables(font)
        .build();
 
    std::fs::write(input_path_str.replace(".ttf", "-mod.ttf"), &new_bytes).unwrap();
}
