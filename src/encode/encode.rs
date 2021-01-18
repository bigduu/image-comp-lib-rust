use crate::io::img_obj::Image;
use crate::encode::clustering::gen_point_cloud::{gen_euclid_cloud, gen_point_cloud};
use crate::encode::clustering::clustering_methods::kmeans_clustering;
use crate::encode::grid::grid_ops::{calc_cluster_map, calc_grid, calc_cluster_colors, calc_data_lists};
use crate::encode::flatten::lists::{flatten_list, bytes_list};
use crate::encode::compress::deflate::deflate;
use crate::encode::flatten::cluster_colors::flatten_cc;
use bytes::{Bytes, BufMut};

pub fn comp_img(img: &Image, b_size: usize, k_n: usize) -> Bytes {
    let cloud = gen_euclid_cloud(img);
    let cluster = kmeans_clustering(&cloud, k_n);
    let points = gen_point_cloud(&img);
    let dims = &img.dim();
    let cluster_map = calc_cluster_map(&cluster, &points, *dims);
    let grid = calc_grid(&cluster_map, b_size);
    let cluster_colors = calc_cluster_colors(&cluster, &points);
    let lists = calc_data_lists(&img, &grid, &cluster_colors);
    let norm = &lists[0];
    let abs = &lists[1];
    let norm_f = flatten_list(norm);
    let abs_f = flatten_list(abs);
    let norm_b = bytes_list(&norm_f);
    let abs_b = bytes_list(&abs_f);
    let norm_c = deflate(&norm_b);
    let abs_c = deflate(&abs_b);
    let gl = grid.to_list();
    let gl_b = bytes_list(&gl);
    let gl_c = deflate(&gl_b);
    let cc = flatten_cc(&cluster_colors);
    let cc_b = bytes_list(&cc);
    let parts = [gl_c, cc_b, norm_c, abs_c];
    let mut buf = vec![];
    for p in parts.iter() {
        let len = p.len() as u64;
        let obj = p.clone();
        buf.put_u64(len);
        buf.put(obj);
    }
    Bytes::from(buf)
}