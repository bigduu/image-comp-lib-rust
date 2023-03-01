use std::path::PathBuf;

use image::{ImageFormat, open};
use ndarray::Array3;
use nshare::ToNdarray3;

use crate::io::img_obj::Image;

pub fn load_image(p: PathBuf) -> Image {
    let img = open(p).unwrap().into_rgb8();
    let data: Array3<u8> = img.clone().to_ndarray3();
    Image { img, data }
}

pub fn load_image_vec(vec: Vec<u8>) -> Image {
    let img = image::load_from_memory(&vec).unwrap().into_rgb8();
    let data: Array3<u8> = img.clone().to_ndarray3();
    Image { img, data }
}
