
#[macro_use]
extern crate smallvec;
extern crate rand;
extern crate half;

use std::convert::TryInto;
use std::io::BufWriter;
use std::fs::File;

// exr imports
extern crate exr;

/// Generate a striped image on the fly and directly write that to a file without allocating the whole image at once.
/// On my machine, this program produces a 3GB file while only ever allocating 4MB memory (takes a while though).
fn main() {
    use exr::prelude::*;
    use attribute::*;
    use exr::math::*;

    // TODO implement this example using the new API and not the raw function interface.


    // pre-compute a list of random values
    let random_values: Vec<f32> = (0..64)
        .map(|_| rand::random::<f32>())
        .collect();

    // resulting resolution (268 megapixels for 3GB files)
    let size = (2048*8, 2048*8);

    // specify output path, and buffer it for better performance
    let file = BufWriter::new(File::create("tests/images/out/3GB.exr").unwrap());

    // define meta data header that will be written
    let header = exr::meta::header::Header::new(
        "test-image".try_into().unwrap(),
        size,
        smallvec![
            attribute::ChannelDescription::new("B", SampleType::F32, true),
            attribute::ChannelDescription::new("G", SampleType::F32, true),
            attribute::ChannelDescription::new("R", SampleType::F32, true),
            attribute::ChannelDescription::new("Z", SampleType::F32, true),
        ],
    );

    // define encoding that will be written
    let mut header = header.with_encoding(
        Compression::Uncompressed,

        exr::meta::Blocks::Tiles(TileDescription {
            tile_size: Vec2(64, 64),
            level_mode: LevelMode::Singular,
            rounding_mode: RoundingMode::Down
        }),

        LineOrder::Increasing
    );

    // add some random meta data
    header.own_attributes.exposure = Some(1.0);


    let headers = smallvec![ header ];

    // print progress only every 100th time
    let start_time = ::std::time::Instant::now();

    // finally write the image
    exr::block::lines::write_all_lines_to_buffered(
        file,
        headers,

        // fill the image file contents with one of the precomputed random values,
        // picking a different one per channel
        |_meta, line_mut|{
            let chan = line_mut.location.channel;

            if chan == 3 { // write time as depth (could also do _meta.channels[chan].name == "Z")
                line_mut.write_samples(|_| start_time.elapsed().as_secs_f32())
                    .expect("write to line bug");
            }

            else { // write rgba color
                line_mut
                    .write_samples(|sample_index| random_values[(sample_index + chan) % random_values.len()])
                    .expect("write to line bug");
            }
        },

        |progress|{
            println!("progress: {:.2}%", progress*100.0);
        },

        true,
        false
    ).unwrap();

    // warning: highly unscientific benchmarks ahead!
    println!("\ncreated file 3GB.exr in {:?}s", start_time.elapsed().as_secs_f32());
}