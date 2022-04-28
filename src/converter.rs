use image::RgbaImage;
use visioncortex::color_clusters::{Runner, RunnerConfig, HIERARCHICAL_MAX};
use visioncortex::{Color, ColorImage, ColorName};

use super::config::{ColorMode, Config, ConverterConfig, Hierarchical};
use super::svg::SvgFile;

/// Convert an image file into svg file
pub fn convert_image_to_svg(config: Config, image: RgbaImage) -> Result<String, String> {
    let config = config.into_converter_config();
    match config.color_mode {
        ColorMode::Color => color_image_to_svg(config, image),
        ColorMode::Binary => binary_image_to_svg(config, image),
    }
}

fn color_image_to_svg(config: ConverterConfig, image: RgbaImage) -> Result<String, String> {
    let (img, width, height);
    match prepare_image(image) {
        Ok(values) => {
            img = values.0;
            width = values.1;
            height = values.2;
        }
        Err(msg) => return Err(msg),
    }

    let runner = Runner::new(
        RunnerConfig {
            diagonal: config.layer_difference == 0,
            hierarchical: HIERARCHICAL_MAX,
            batch_size: 25600,
            good_min_area: config.filter_speckle_area,
            good_max_area: (width * height),
            is_same_color_a: config.color_precision_loss,
            is_same_color_b: 1,
            deepen_diff: config.layer_difference,
            hollow_neighbours: 1,
        },
        img,
    );

    let mut clusters = runner.run();

    match config.hierarchical {
        Hierarchical::Stacked => {}
        Hierarchical::Cutout => {
            let view = clusters.view();
            let image = view.to_color_image();
            let runner = Runner::new(
                RunnerConfig {
                    diagonal: false,
                    hierarchical: 64,
                    batch_size: 25600,
                    good_min_area: 0,
                    good_max_area: (image.width * image.height) as usize,
                    is_same_color_a: 0,
                    is_same_color_b: 1,
                    deepen_diff: 0,
                    hollow_neighbours: 0,
                },
                image,
            );
            clusters = runner.run();
        }
    }

    let view = clusters.view();

    let mut svg = SvgFile::new(width, height, config.path_precision);
    for &cluster_index in view.clusters_output.iter().rev() {
        let cluster = view.get_cluster(cluster_index);
        let paths = cluster.to_compound_path(
            &view,
            false,
            config.mode,
            config.corner_threshold,
            config.length_threshold,
            config.max_iterations,
            config.splice_threshold,
        );
        svg.add_path(paths, cluster.residue_color());
    }

    Ok(svg.to_string())
}

fn binary_image_to_svg(config: ConverterConfig, image: RgbaImage) -> Result<String, String> {
    let (img, width, height);
    match prepare_image(image) {
        Ok(values) => {
            img = values.0;
            width = values.1;
            height = values.2;
        }
        Err(msg) => return Err(msg),
    }
    let img = img.to_binary_image(|x| x.r < 128);

    let clusters = img.to_clusters(false);

    let mut svg = SvgFile::new(width, height, config.path_precision);
    for i in 0..clusters.len() {
        let cluster = clusters.get_cluster(i);
        if cluster.size() >= config.filter_speckle_area {
            let paths = cluster.to_compound_path(
                config.mode,
                config.corner_threshold,
                config.length_threshold,
                config.max_iterations,
                config.splice_threshold,
            );
            svg.add_path(paths, Color::color(&ColorName::Black));
        }
    }

    Ok(svg.to_string())
}

fn prepare_image(img: RgbaImage) -> Result<(ColorImage, usize, usize), String> {
    let (width, height) = (img.width() as usize, img.height() as usize);
    let img = ColorImage {
        pixels: img.as_raw().to_vec(),
        width,
        height,
    };

    Ok((img, width, height))
}
