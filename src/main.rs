use svg_trace::{convert_image_to_svg, Config, Preset};

static IMAGE_BYTES: &[u8] = include_bytes!("../tc.png");

fn main() {
    match convert_image_to_svg(Config::from_preset(Preset::Bw), IMAGE_BYTES) {
        Ok(svg) => {
            println!("{svg}");
        }
        Err(msg) => {
            eprintln!("Conversion failed with error message: {}", msg);
        }
    }
}
