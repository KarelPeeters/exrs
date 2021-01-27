
extern crate image as png;

extern crate exr;


/// Read an rgba image, crop away transparent pixels,
/// then write the cropped result to another file.
pub fn main() {
    use exr::prelude::*;
    use exr::image::read::specific_channels::pixels::*;

    let path = "tests/images/valid/custom/oh crop.exr";

    // load an rgba image
    // this specific example discards all but the first valid rgb layers and converts all pixels to f32 values
    // TODO optional alpha channel!
    let image: PixelImage<Flattened<(Sample, Sample, Sample, Option<Sample>)>, _> = read_first_rgba_layer_from_file(
        path,
        create_flattened, // ::<(Sample, Sample, Sample, Sample)>,
        set_flattened_pixel // use some predefined rgba pixel vector
    ).unwrap();

    // construct a ~simple~ cropped image
    let image: Image<Layer<CroppedChannels<SpecificChannels<Flattened<(Sample, Sample, Sample, Option<Sample>)>, _>>>> = Image {
        attributes: image.attributes,

        // crop each layer
        layer_data: {
            println!("cropping layer {:#?}", image.layer_data);

            // if has alpha, crop it where alpha is zero
            image.layer_data
                .crop_where(|(_r, _g, _b, alpha)|
                    match alpha {
                        None => false, // never crop images without alpha channel
                        Some(alpha) => alpha.is_zero(),
                    }
                )
                .or_crop_to_1x1_if_empty() // do not remove empty layers from image, because it could result in an image without content
        },
    };

    image.write().to_file("tests/images/out/cropped_rgba.exr").unwrap();
    println!("cropped file to cropped_rgba.exr");
}

