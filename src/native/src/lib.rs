use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::os::raw::c_char;
use std::mem;
use std::ffi::{c_void, CStr};
use std::path::Path;
use std::slice::from_raw_parts;

use exr::error::UnitResult;
use exr::prelude::*;

macro_rules! unwrap_or_return_err {
    ($e: expr) => {
        match $e {
            Ok(e) => e,
            Err(err) => {
                println!("{err}");
                return 1;
            }
        }
    };
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum ExrEncoding {
    Uncompressed = 0,
    RLE = 1,
    ZIP1 = 2,
    ZIP16 = 3,
    PIZ = 4,
}

#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum ExrPixelFormat
{
    Unknown = -1,
    U32 = 0,
    F16 = 1,
    F32 = 2,
    RGBF32 = 3
}

#[no_mangle]
pub unsafe extern fn write_texture(path: *const c_char, width: i32, height: i32, format: ExrPixelFormat, encoding: ExrEncoding, data: *const Sample) -> i32 {
    let path = match CStr::from_ptr(path).to_str() {
        Ok(path) => path,
        Err(err) => {
            println!("{err}");
            return 1
        }
    };

    let result = match format {
        ExrPixelFormat::U32 => {
            let ptr = data as *const u32;
            let array = from_raw_parts(ptr, (width * height * 4) as usize);
            write_exr(path, array, width as usize, height as usize, encoding)
        },
        ExrPixelFormat::F16 => {
            let ptr = data as *const f16;
            let array = from_raw_parts(ptr, (width * height * 4) as usize);
            write_exr(path, array, width as usize, height as usize, encoding)
        },
        ExrPixelFormat::F32 => {
            let ptr = data as *const f32;
            let array = from_raw_parts(ptr, (width * height * 4) as usize);
            write_exr(path, array, width as usize, height as usize, encoding)
        }
        _ => {
            // Unknown
            Err(Error::NotSupported(Cow::Owned(format!("Encoding {encoding:?} not supported"))))
        }
    };

    match result {
        Ok(()) => 0,
        Err(err) => {
            println!("{err}");
            1
        },
    }
}

fn write_exr<T: IntoSample>(path: impl AsRef<Path>, array: &[T], width: usize, height: usize, encoding: ExrEncoding) -> UnitResult {
    let channels = SpecificChannels::rgba(|Vec2(x,y)| (
        array[(y * width + x) * 4 + 0],
        array[(y * width + x) * 4 + 1],
        array[(y * width + x) * 4 + 2],
        array[(y * width + x) * 4 + 3]
    ));
    let encoding = match encoding  {
        // See encoding presets but expanded here to make clearer the
        // encoding compression
        ExrEncoding::Uncompressed => Encoding {
            compression: Compression::Uncompressed,
            blocks: Blocks::ScanLines, // longest lines, faster memcpy
            line_order: LineOrder::Increasing // presumably fastest?
        },
        ExrEncoding::RLE => Encoding {
            compression: Compression::RLE,
            blocks: Blocks::Tiles(Vec2(64, 64)), // optimize for RLE compression
            line_order: LineOrder::Unspecified
        },
        ExrEncoding::ZIP16 => Encoding {
            compression: Compression::ZIP16,
            blocks: Blocks::ScanLines, // largest possible, but also with high probability of parallel workers
            line_order: LineOrder::Increasing
        },
        ExrEncoding::PIZ => Encoding {
            compression: Compression::PIZ,
            blocks: Blocks::Tiles(Vec2(256, 256)),
            line_order: LineOrder::Unspecified
        },
        ExrEncoding::ZIP1 => Encoding {
            compression: Compression::ZIP1,
            blocks: Blocks::ScanLines,
            line_order: LineOrder::Increasing
        }
    };
    let layer = Layer::new(
        Vec2(width, height),
        LayerAttributes::named("first layer"),
        encoding,
        channels
    );
    Image::from_layer(layer).write().to_file(path)
}

#[no_mangle]
pub unsafe extern fn load_from_path(path: *const c_char, width: *mut i32, height: *mut i32, format: *mut i32, data: *mut *mut c_void) -> i32 {
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(path) => path,
        Err(err) => {
            println!("{err}");
            return 1
        }
    };
    let extension = match Path::new(path_str)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some(extension) => extension,
        None => ""
    };

    match extension {
        "hdr" => {
            let f = unwrap_or_return_err!(File::open(path_str));
            let r = BufReader::new(f);
            let mut image = unwrap_or_return_err!(radiant::load(r));

            *width = image.width as i32;
            *height = image.height as i32;
            *format = 3;

            let ptr = image.data.as_mut_ptr();
            mem::forget(image);

            *data = ptr as *mut c_void;
            0
        },
        _ => {
            match MetaData::read_from_file(path_str, false) {
                Ok(meta) => {
                    let size = meta.headers[0].layer_size;
                    *width = size.0 as i32;
                    *height = size.1 as i32;

                    let sample_type = meta.headers[0].channels.uniform_sample_type;

                    match sample_type {
                        Some(v) => {
                            *format = v as i32;

                            *data = match v {
                                SampleType::F16 => unwrap_or_return_err!(load_exr_f16(path_str)) as *mut c_void,
                                SampleType::F32 => unwrap_or_return_err!(load_exr_f32(path_str)) as *mut c_void,
                                SampleType::U32 => unwrap_or_return_err!(load_exr_u32(path_str)) as *mut c_void,
                            }; 
                        },
                        None => {
                            *format = -1;
                            *data = std::ptr::null_mut() as *mut c_void;
                        }
                    }
                    0
                },
                Err(_e) => {
                    *width = -1;
                    *height = -1;
                    *format = -1;
                    *data = std::ptr::null_mut() as *mut c_void;
                    1
                }
            }
        }
    }
}

fn load_exr_f16(path: &str) -> Result<*mut [f16;4]> {
    let image = read_first_rgba_layer_from_file(
        path,
        |resolution, _| {
            let default_pixel: [f16;4] = [f16::from_f32(0.0), f16::from_f32(0.0), f16::from_f32(0.0), f16::from_f32(1.0)];
            let empty_line = vec![ default_pixel; resolution.width() ];
            let empty_image = vec![ empty_line; resolution.height() ];
            empty_image
        },
        |pixel_vector, position, (r,g,b, a): (f16, f16, f16, f16)| {
            pixel_vector[position.y()][position.x()] = [r, g, b, a]
        },

    )?;

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[f16;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);

    Ok(ptr)
}

fn load_exr_f32(path: &str) -> Result<*mut [f32;4]> {
    let image = read_first_rgba_layer_from_file(
        path,
        |resolution, _| {
            let default_pixel: [f32;4] = [0.0, 0.0, 0.0, 1.0];
            let empty_line = vec![ default_pixel; resolution.width() ];
            let empty_image = vec![ empty_line; resolution.height() ];
            empty_image
        },
        |pixel_vector, position, (r,g,b, a): (f32, f32, f32, f32)| {
            pixel_vector[position.y()][position.x()] = [r, g, b, a]
        },

    )?;

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[f32;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);

    Ok(ptr)
}

fn load_exr_u32(path: &str) -> Result<*mut [u32;4]> {
    let image = read_first_rgba_layer_from_file(
        path,
        |resolution, _| {
            let default_pixel: [u32;4] = [0, 0, 0, 1];
            let empty_line = vec![ default_pixel; resolution.width() ];
            let empty_image = vec![ empty_line; resolution.height() ];
            empty_image
        },
        |pixel_vector, position, (r,g,b, a): (u32, u32, u32, u32)| {
            pixel_vector[position.y()][position.x()] = [r, g, b, a]
        },

    )?;

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[u32;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);

    Ok(ptr)
}

// The use of exr::Sample is stored in memory at compile time according to the largest element, f32

// fn load_exr(path: &str) -> usize {
//     let image = read_first_rgba_layer_from_file(
//         path,
//         |resolution, _| {
//             let default_pixel: [Sample;4] = [Sample::default(), Sample::default(), Sample::default(), Sample::default()];
//             let empty_line = vec![ default_pixel; resolution.width() ];
//             let empty_image = vec![ empty_line; resolution.height() ];
//             empty_image
//         },
//         |pixel_vector, position, (r,g,b, a): (Sample, Sample, Sample, Sample)| {
//             pixel_vector[position.y()][position.x()] = [r, g, b, a]
//         },

//     ).unwrap();

//     let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[Sample;4]>>();
//     let ptr = pixel.as_mut_ptr();
//     mem::forget(pixel);

//     return ptr as usize;
// }