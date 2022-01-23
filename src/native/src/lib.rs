use std::os::raw::c_char;
use std::mem;
use std::ffi::CStr;

use exr::prelude::*;
use exr::prelude::f16 as half;

#[no_mangle]
pub unsafe extern fn load_meta_data(path: *const c_char, width: *mut i32, height: *mut i32, format: *mut i32) {
    let path_str = CStr::from_ptr(path).to_str().unwrap();

    match MetaData::read_from_file(path_str, false) {
        Ok(meta) => {
            let size = meta.headers[0].layer_size;
            *width = size.0 as i32;
            *height = size.1 as i32;
        
            let sample_type = meta.headers[0].channels.uniform_sample_type;
        
            match sample_type {
                Some(v) => *format = v as i32,
                None => *format = -1
            }
        },
        Err(_e) => {
            *width = -1;
            *height = -1;
            *format = -1;
        }
    }
}

#[no_mangle]
pub unsafe extern fn load_exr_f32(path: *const c_char) -> *mut [f32;4] {
    let path_str = CStr::from_ptr(path).to_str().unwrap();

    let image = read_first_rgba_layer_from_file(
        path_str,
        |resolution, _| {
            let default_pixel = [0.0, 0.0, 0.0, 0.0];
            let empty_line =  vec![ default_pixel; resolution.width() ];
            let empty_image =  vec![ empty_line; resolution.height() ];
            empty_image
        },
        |pixel_vector, position, (r,g,b, a): (f32, f32, f32, f32)| {
            pixel_vector[position.y()][position.x()] = [r, g, b, a]
        },

    ).unwrap();

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[f32;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);
    
    return ptr;
}

#[no_mangle]
pub unsafe extern fn load_exr_f16(path: *const c_char) -> *mut [half;4] {
    let path_str = CStr::from_ptr(path).to_str().unwrap();

    let image = read_first_rgba_layer_from_file(
        path_str,
        |resolution, _| {
            let default_pixel: [half;4] = [half::default(), half::default(), half::default(), half::default()];
            let empty_line =  vec![ default_pixel; resolution.width() ];
            let empty_image =  vec![ empty_line; resolution.height() ];
            empty_image
        },
        |pixel_vector, position, (r,g,b, a): (half, half, half, half)| {
            pixel_vector[position.y()][position.x()] = [r, g, b, a]
        },

    ).unwrap();

    let mut pixel = image.layer_data.channel_data.pixels.into_iter().flatten().collect::<Vec<[half;4]>>();
    let ptr = pixel.as_mut_ptr();
    mem::forget(pixel);
    
    return ptr;
}
