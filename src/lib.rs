#![allow(dead_code)]

/// Library of the Image Compression Algorithm. Explanation on how it works is available at GitHub:  [ReadMe](https://github.com/umgefahren/image-comp-lib-rust/blob/main/README.md)

#[macro_use]
#[doc(hidden)]
pub mod io;
#[doc(hidden)]
pub mod decode;
#[doc(hidden)]
pub mod encode;

mod debug;

use crate::decode::decoder::con_img;
use crate::encode::encode::comp_img;
use crate::io::load_image::{load_image, load_image_vec};
use bytes::Bytes;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use crate::debug::compare_img;
    use crate::decode::compress::compressors::dec_comp_data;
    use crate::decode::construct::cluster_colors::create_cluster_colors;
    use crate::decode::construct::lists::{create_list, list_f_bytes};
    use crate::decode::decoder::con_img;
    use crate::decode::lists::decode;
    use crate::encode::clustering::clustering_methods::kmeans_clustering;
    use crate::encode::clustering::gen_point_cloud::{gen_euclid_cloud, gen_point_cloud};
    use crate::encode::compress::compressors::comp_data;
    use crate::encode::encode::comp_img;
    use crate::encode::flatten::cluster_colors::flatten_cc;
    use crate::encode::flatten::lists::{bytes_list, flatten_list};
    use crate::encode::grid::grid_obj::from_list;
    use crate::encode::grid::grid_ops::{
        calc_cluster_colors, calc_cluster_map, calc_data_lists, calc_grid,
    };
    use crate::io::load_image::load_image;
    use image::{ImageResult, Rgb, RgbImage};
    use ndarray::Array;
    use std::fs;
    use std::iter::FromIterator;
    use std::path::PathBuf;

    #[test]
    fn clustering_test_no_loss() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let pixel_count = img.img.dimensions().0 as usize * img.img.dimensions().1 as usize;
        let cloud = crate::encode::clustering::gen_point_cloud::gen_euclid_cloud(&img);
        let cluster = crate::encode::clustering::clustering_methods::kmeans_clustering(&cloud, 3);
        let mut size: usize = 0;
        for c in cluster.iter() {
            size += c.len();
        }
        assert_eq!(size, pixel_count);
    }

    #[test]
    #[ignore]
    fn clustering_test_debug() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = crate::encode::clustering::gen_point_cloud::gen_euclid_cloud(&img);
        let points = crate::encode::clustering::gen_point_cloud::gen_point_cloud(&img);
        let cluster = crate::encode::clustering::clustering_methods::kmeans_clustering(&cloud, 3);
        let mut debug_img = RgbImage::new(img.img.dimensions().0, img.img.dimensions().1);
        for (_idx, c) in cluster.iter().enumerate() {
            let mut add_color: [u32; 3] = [0, 0, 0];
            let mut counter: u32 = 1;
            for p in c {
                counter += 1;
                add_color[0] = add_color[0] + points[*p][2];
                add_color[1] = add_color[1] + points[*p][3];
                add_color[2] = add_color[2] + points[*p][4];
            }
            add_color[0] = add_color[0] / counter;
            add_color[1] = add_color[1] / counter;
            add_color[2] = add_color[2] / counter;
            let add_color = [add_color[0] as u8, add_color[1] as u8, add_color[2] as u8];
            let def_color: Rgb<u8> = Rgb(add_color);
            /*
            if idx == 0 {
                def_color = Rgb([255, 0, 0]);
            } else if idx == 1 {
                def_color = Rgb([0, 255, 0]);
            } else if idx == 2 {
                def_color = Rgb([0, 0, 255]);
            } else if idx == 3 {
                def_color = Rgb([255, 255, 0]);
            } else if idx == 4 {
                def_color = Rgb([255, 0, 255]);
            } else if idx == 5 {
                def_color = Rgb([0, 255, 255]);
            } else if idx == 6 {
                def_color = Rgb([255, 255, 255]);
            } else if idx == 7 {
                def_color = Rgb([0, 0, 0]);
            }
            */
            for p in c.iter() {
                debug_img.put_pixel(points[*p][0] as u32, points[*p][1] as u32, def_color);
            }
        }

        #[cfg(feature = "debug-out-img")]
        debug_img.save("./images/out.png");
    }

    #[test]
    #[ignore]
    fn cluster_map_test() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = crate::encode::clustering::gen_point_cloud::gen_euclid_cloud(&img);
        let cluster = crate::encode::clustering::clustering_methods::kmeans_clustering(&cloud, 3);
        let points = crate::encode::clustering::gen_point_cloud::gen_point_cloud(&img);
        let dims = img.dim();
        let ret = crate::encode::grid::grid_ops::calc_cluster_map(&cluster, &points, dims);
        let mut debug_img = RgbImage::new(img.img.dimensions().0, img.img.dimensions().1);
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                let val = (ret[[x, y]] * 10) as u8;
                debug_img.put_pixel(x as u32, y as u32, Rgb([val, val, val]))
            }
        }
        #[cfg(feature = "debug-out-img")]
        debug_img.save("./images/out_cluster_map.png").unwrap();
        assert_eq!(dims, ret.shape());
    }

    #[test]
    #[ignore]
    fn grid_map_test_and_debug() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = crate::encode::clustering::gen_point_cloud::gen_euclid_cloud(&img);
        let cluster = crate::encode::clustering::clustering_methods::kmeans_clustering(&cloud, 3);
        let points = crate::encode::clustering::gen_point_cloud::gen_point_cloud(&img);
        let dims = &img.dim();
        let cluster_map = crate::encode::grid::grid_ops::calc_cluster_map(&cluster, &points, *dims);
        let grid = crate::encode::grid::grid_ops::calc_grid(&cluster_map, 10);
        let img_out_grid = grid.render();
        #[cfg(feature = "debug-out-img")]
        img_out_grid.save("./images/out_grid.png").unwrap();
        assert_eq!(*dims, grid.image_dim());
    }

    #[test]
    #[ignore]
    fn grid_debug_img() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let mut res_img = RgbImage::new(img.img.dimensions().0, img.img.dimensions().1);
        let cloud = crate::encode::clustering::gen_point_cloud::gen_euclid_cloud(&img);
        let cluster = crate::encode::clustering::clustering_methods::kmeans_clustering(&cloud, 3);
        let points = crate::encode::clustering::gen_point_cloud::gen_point_cloud(&img);
        let dims = &img.dim();
        let cluster_map = crate::encode::grid::grid_ops::calc_cluster_map(&cluster, &points, *dims);
        let grid = crate::encode::grid::grid_ops::calc_grid(&cluster_map, 10);
        let cluster_colors = calc_cluster_colors(&cluster, &points);
        let w_len = grid.w * grid.wx + 1;
        let h_len = grid.h * grid.hx + 1;
        for idy in 0..grid.hx {
            // println!("{:?}", x_arr);
            for idx in 0..grid.wx {
                // let mut _real_w = grid.w;
                // let mut _real_h = grid.h;
                // if ((idx + 1) * grid.w) >= w_len {
                //     real_w -= 1;
                // }
                // if ((idy + 1) * grid.h) >= h_len {
                //     real_h -= 1;
                // }
                let code = &grid.data[[idx, idy]].to_owned();
                let base_color = if *code == 16 {
                    Rgb([255 as u8, 255 as u8, 255 as u8])
                } else {
                    Rgb(cluster_colors.get(code).unwrap().to_owned())
                };
                let mut counter = 0;
                // println!("Idxe: {} Idye: {}", (idx + 1) * grid.w, (idy + 1) * grid.h);
                for x in (idx * grid.w)..((idx + 1) * grid.w) {
                    if x >= w_len - 1 {
                        continue;
                    }
                    let y_iter: Vec<usize> = if counter % 2 == 0 {
                        ((idy * grid.h)..(idy + 1) * grid.h).collect()
                    } else {
                        ((idy * grid.h)..(idy + 1) * grid.h).rev().collect()
                    };
                    counter += 1;
                    // let y_iter: Vec<usize> = ((idy * grid.h)..(idy + 1) * grid.h).collect();
                    for y in y_iter {
                        // println!("x: {} idx: {} y: {} idy: {} {:?}", x, idx, y, idy, res_img.dimensions());
                        if y >= h_len - 1 {
                            continue;
                        }
                        res_img.put_pixel(x as u32, y as u32, base_color);
                    }
                }
            }
        }
        #[cfg(feature = "debug-out-img")]
        res_img.save("images/cluster_color_grid.png").unwrap();
    }

    #[test]
    #[ignore]
    fn points_test_debug() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let points = crate::encode::clustering::gen_point_cloud::gen_point_cloud(&img);
        let mut debug_img = RgbImage::new(img.img.dimensions().0, img.img.dimensions().1);
        for p in points {
            debug_img.put_pixel(p[0], p[1], Rgb([p[2] as u8, p[3] as u8, p[4] as u8]));
        }
        #[cfg(feature = "debug-out-img")]
        debug_img.save("./images/out_points.png").unwrap();
    }

    #[test]
    fn lists_test() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = crate::encode::clustering::gen_point_cloud::gen_euclid_cloud(&img);
        let cluster = crate::encode::clustering::clustering_methods::kmeans_clustering(&cloud, 3);
        let points = crate::encode::clustering::gen_point_cloud::gen_point_cloud(&img);
        let dims = &img.dim();
        let cluster_map = crate::encode::grid::grid_ops::calc_cluster_map(&cluster, &points, *dims);
        let grid = crate::encode::grid::grid_ops::calc_grid(&cluster_map, 10);
        let cluster_colors = crate::encode::grid::grid_ops::calc_cluster_colors(&cluster, &points);
        let lists = crate::encode::grid::grid_ops::calc_data_lists(&img, &grid, &cluster_colors);
        let norm = &lists[0];
        let abs = &lists[1];
        let norm_f = flatten_list(norm);
        let abs_f = flatten_list(abs);
        let norm_r = create_list(norm_f);
        let abs_r = create_list(abs_f);
        let res_lists = [norm_r, abs_r];
        assert_eq!(lists, res_lists);
    }

    #[test]
    #[ignore]
    fn list_debug() {
        let p = PathBuf::from("./images/img_1.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = crate::encode::clustering::gen_point_cloud::gen_euclid_cloud(&img);
        let cluster = crate::encode::clustering::clustering_methods::kmeans_clustering(&cloud, 3);
        let points = crate::encode::clustering::gen_point_cloud::gen_point_cloud(&img);
        let dims = &img.dim();
        let cluster_map = crate::encode::grid::grid_ops::calc_cluster_map(&cluster, &points, *dims);
        let grid = crate::encode::grid::grid_ops::calc_grid(&cluster_map, 100);
        let cluster_colors = crate::encode::grid::grid_ops::calc_cluster_colors(&cluster, &points);
        let lists = crate::encode::grid::grid_ops::calc_data_lists(&img, &grid, &cluster_colors);
        let norm = &lists[0];
        let abs = &lists[1];
        let norm_f = flatten_list(norm);
        let abs_f = flatten_list(abs);
        let norm_r = create_list(norm_f);
        let abs_r = create_list(abs_f);
        let res_lists = [norm_r, abs_r];
        let res_img = decode(&res_lists[0], &res_lists[1], &grid, &cluster_colors);
        #[cfg(feature = "debug-out-img")]
        res_img.save("./images/decoded_list.png").unwrap();
        compare_img(&img.img, &res_img);
        assert_eq!(img.img, res_img);
    }
    #[test]
    fn deflate_test() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = gen_euclid_cloud(&img);
        let cluster = kmeans_clustering(&cloud, 3);
        let points = gen_point_cloud(&img);
        let dims = &img.dim();
        let cluster_map = calc_cluster_map(&cluster, &points, *dims);
        let grid = calc_grid(&cluster_map, 100);
        let cluster_colors = calc_cluster_colors(&cluster, &points);
        let lists = calc_data_lists(&img, &grid, &cluster_colors);
        let norm = &lists[0];
        let abs = &lists[1];
        let norm_f = flatten_list(norm);
        let bytes_n = bytes_list(&norm_f);
        let comp_n = comp_data(&bytes_n);
        let decomp_n = dec_comp_data(&comp_n);
        let abs_f = flatten_list(abs);
        let bytes_a = bytes_list(&abs_f);
        let comp_a = comp_data(&bytes_a);
        let decomp_a = dec_comp_data(&comp_a);
        let decomp_n_l = list_f_bytes(&decomp_n);
        let decomp_a_l = list_f_bytes(&decomp_a);
        let norm_r = create_list(decomp_n_l);
        let abs_r = create_list(decomp_a_l);
        let res_lists = [norm_r, abs_r];
        println!("Uncompressed Size: {}", bytes_n.len() + bytes_a.len());
        println!("Compressed size:   {}", comp_n.len() + comp_a.len());
        let res_img = decode(&res_lists[0], &res_lists[1], &grid, &cluster_colors);
        #[cfg(feature = "debug-out-img")]
        res_img.save("./images/decoded_comp_list.png").unwrap();
        assert_eq!(lists, res_lists);
    }
    #[test]
    fn deflate_grid() {
        let p = PathBuf::from("./images/img_2.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = gen_euclid_cloud(&img);
        let cluster = kmeans_clustering(&cloud, 3);
        let points = gen_point_cloud(&img);
        let dims = &img.dim();
        let cluster_map = calc_cluster_map(&cluster, &points, *dims);
        let grid = calc_grid(&cluster_map, 10);
        let g_list = grid.to_list();
        let g_b_list = bytes_list(&g_list);
        let comp_gb_list = comp_data(&g_b_list);
        println!("Uncompressed size: {}", g_b_list.len());
        println!("Compressed size:   {}", comp_gb_list.len());
        let dec_comp_gb_list = dec_comp_data(&comp_gb_list);
        let dec_g_list = list_f_bytes(&dec_comp_gb_list);
        let grid_n = from_list(&dec_g_list);
        assert_eq!(grid, grid_n);
    }

    #[test]
    fn flatten_cluster_colors() {
        let p = PathBuf::from("./images/img_1.png");
        let img = crate::io::load_image::load_image(p);
        let cloud = gen_euclid_cloud(&img);
        let cluster = kmeans_clustering(&cloud, 15);
        let points = gen_point_cloud(&img);
        let cluster_colors = calc_cluster_colors(&cluster, &points);
        let flat_cc = flatten_cc(&cluster_colors);
        let cc_b = bytes_list(&flat_cc);
        let comp_cc_b = comp_data(&cc_b);
        println!("Uncompressed Size: {}", cc_b.len());
        println!("Compressed Size:   {}", comp_cc_b.len());
        let ret_cc = create_cluster_colors(&flat_cc);
        assert_eq!(cluster_colors, ret_cc);
    }

    #[test]
    fn encode_test() {
        let p = PathBuf::from("./images/img_4.png");
        let org_len = fs::metadata(&p).unwrap().len();
        let img = crate::io::load_image::load_image(p);
        let bs = comp_img(&img, 10, 3);
        println!("Compressed Img Size {} Bytes", bs.len());
        println!("Original Img Size   {} Bytes", org_len);
        let img2 = con_img(&bs);
        #[cfg(feature = "debug-out-img")]
        img2.save("./images/decompressed.png").unwrap();
        assert_eq!(img2, img.img)
    }

    #[test]
    fn encode_raw_values() {
        let p = PathBuf::from("./images/img_4.png");
        let img = load_image(p.clone());
        let flat: Vec<u8> = Array::from_iter(img.data.iter())
            .iter()
            .map(|i| i.to_owned().to_owned())
            .collect();
        let b = bytes_list(&flat);
        let comp = comp_data(&b);
        println!("Compressed Flat image: {}", comp.len());
        let other_comp = comp_img(&img, 10, 3);
        println!("Compressed image:      {}", other_comp.len());
        let org_len = fs::metadata(&p).unwrap().len();
        println!("Org Image:             {}", org_len);
    }
}

fn write_f(target_p: PathBuf, data: &Bytes) {
    let mut file = File::create(target_p).unwrap();
    file.write_all(data).unwrap();
}

fn read_f(target: PathBuf) -> Bytes {
    let mut buffer = Vec::new();
    let mut f = File::open(target).unwrap();
    f.read_to_end(&mut buffer).unwrap();
    Bytes::from(buffer)
}

/// Compress Image
/// 1. Input --> String describing the input path
/// 2. Input --> String describing the output path
pub fn compress_image(org_s: &String, target_s: &String) {
    let org_p = PathBuf::from(org_s);
    let target_p = PathBuf::from(target_s);
    let img = load_image(org_p);
    let bs = comp_img(&img, 10, 3);
    write_f(target_p, &bs);
}

/// Compress Image Vec
/// 1. Input --> The image as a vector
/// 2. Output --> The compressed image as a vector
pub fn compress_image_vec(vec: Vec<u8>) -> Vec<u8> {
    let img = load_image_vec(vec);
    let bs = comp_img(&img, 10, 3);
    bs.to_vec()
}

/// Decompress Image
/// 1. Input --> String describing the input path
/// 2. Input --> String describing the output path
pub fn decompress_image(org_s: &String, target_s: &String) {
    let org_p = PathBuf::from(org_s);
    let target_p = PathBuf::from(target_s);
    let bs = read_f(org_p);
    let img = con_img(&bs);
    img.save(target_p).unwrap();
}
