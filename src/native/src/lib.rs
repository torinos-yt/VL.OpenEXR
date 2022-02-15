use std::fs::File;
use std::io::BufReader;
use std::os::raw::c_char;
use std::mem;
use std::ffi::CStr;
use std::path::Path;
use std::slice::from_raw_parts;

use exr::prelude::*;

#[no_mangle]
pub unsafe extern fn write_texture(path: *const c_char, width: i32, height: i32, format: i32, data: *const Sample) {
    let path_str = CStr::from_ptr(path).to_str().unwrap();

    match format {
        0 => { // U32
            let ptr = data as *const u32;
            let array = from_raw_parts(ptr, (width * height * 4) as usize);
            write_rgba_file(
                path_str,
                width as usize, height as usize,
                |x,y| (
                    array[(y * (width as usize) + x) * 4 + 0],
                    array[(y * (width as usize) + x) * 4 + 1],
                    array[(y * (width as usize) + x) * 4 + 2],
                    array[(y * (width as usize) + x) * 4 + 3]
                )
            ).unwrap();
        },
        1 => { // F16
            let ptr = data as *const f16;
            let array = from_raw_parts(ptr, (width * height * 4) as usize);
            write_rgba_file(
                path_str,
                width as usize, height as usize,
                |x,y| (
                    array[(y * (width as usize) + x) * 4 + 0],
                    array[(y * (width as usize) + x) * 4 + 1],
                    array[(y * (width as usize) + x) * 4 + 2],
                    array[(y * (width as usize) + x) * 4 + 3]
                )
            ).unwrap();
        },
        2 => { // F32
            let ptr = data as *const f32;
            let array = from_raw_parts(ptr, (width * height * 4) as usize);
            write_rgba_file(
                path_str,
                width as usize, height as usize,
                |x,y| (
                    array[(y * (width as usize) + x) * 4 + 0],
                    array[(y * (width as usize) + x) * 4 + 1],
                    array[(y * (width as usize) + x) * 4 + 2],
                    array[(y * (width as usize) + x) * 4 + 3]
                )
            ).unwrap();
        }
        _ => { // Unknown
        }
    }
}

#[no_mangle]
pub unsafe extern fn load_from_path(path: *const c_char, width: *mut i32, height: *mut i32, format: *mut i32) -> *mut [Sample;4] {
    let path_str = CStr::from_ptr(path).to_str().unwrap();
    let extension = Path::new(path_str).extension().unwrap().to_str().unwrap();

    match extension {
        "hdr" => {
            let r = BufReader::new(File::open(path_str).unwrap());
            let mut image = radiant::load(r).unwrap();

            *width = image.width as i32;
            *height = image.height as i32;
            *format = 3;

            let ptr = image.data.as_mut_ptr();
            mem::forget(image);

            ptr as *mut [Sample;4]
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
        
                            match v {
                                SampleType::F16 => load_exr_f16(path_str) as *mut [Sample;4],
                                SampleType::F32 => load_exr_f32(path_str) as *mut [Sample;4],
                                SampleType::U32 => load_exr_u32(path_str) as *mut [Sample;4]
                            }
                        },
                        None => {
                            *format = -1;
                            std::ptr::null_mut() as *mut [Sample;4]
                        }
                    }
                },
                Err(_e) => {
                    *width = -1;
                    *height = -1;
                    *format = -1;
        
                    std::ptr::null_mut() as *mut [Sample;4]
                }
            }
        }
    }
}

fn load_exr_f16(path: &str) -> usize {
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

    ).unwrap();

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[f16;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);
    
    return unsafe { mem::transmute(ptr) };
}

fn load_exr_f32(path: &str) -> usize {
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

    ).unwrap();

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[f32;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);
    
    return unsafe { mem::transmute(ptr) };
}

fn load_exr_u32(path: &str) -> usize {
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

    ).unwrap();

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[u32;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);
    
    return unsafe { mem::transmute(ptr) };
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
    
//     return unsafe { mem::transmute(ptr) };
// }
