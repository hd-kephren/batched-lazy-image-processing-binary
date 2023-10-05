use std::ffi::OsStr;
use std::fs::{DirEntry, File};
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

use fraction::{Fraction, ToPrimitive};
use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use rayon::prelude::*;
use regex::Regex;

use crate::imports::directory_to_files;
use crate::structs::Args;

use std::sync::atomic::Ordering;
use atomic_float::AtomicF32;
use image::codecs::png::PngEncoder;

pub fn process_images(args: &Args, progress: &'static AtomicF32) {
    let batch_size = args.batch_size;
    let input = args.input.as_str();
    let extensions: Vec<&str> = args.extensions.split("|").collect();
    let filtered_files = directory_to_files(&input, &extensions);
    let count = filtered_files.iter().count();
    let steps = 1.0 / count as f32;

    filtered_files
        .chunks(batch_size)
        .for_each(|filtered_files_of_files| {
            filtered_files_of_files
                .par_iter()
                .for_each(|file| {
                    let x = progress.load(Ordering::SeqCst);
                    progress.swap(x + steps, Ordering::SeqCst);
                    process_image(file, &args);
                })
        });
    progress.swap(1.0, Ordering::SeqCst);
}

pub fn process_image(file: &std::io::Result<DirEntry>, args: &Args) {
    let path = file.as_ref().unwrap().path();
    process_image_from_path(&path, args);
}

pub fn process_image_from_path(path: &PathBuf, args: &Args) {
    let file_extension = path.extension().and_then(OsStr::to_str);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let _result = match file_extension {
        None => (),
        Some("jpg" | "jpeg") => process_jpg(path, args),
        Some("png") => process_png(path, args),
        Some(ext) => {
            println!("{} | Image format '{}' not supported.", file_name, ext)
        }
    };
}

pub fn process_in_memory_image(image: &Option<DynamicImage>, args: &Args) -> Vec<u8> {
    match image {
        Some(img) => {
            let current_aspect = Fraction::from(img.width()) / Fraction::from(img.height());
            let img = &crop_image(img, current_aspect, args.aspect_ratio);
            let img = &resize_image(img, args.max_width);
            let inner = Vec::new();
            let mut buff = BufWriter::new(inner);
            let encoder = JpegEncoder::new_with_quality(&mut buff, args.quality);
            let _result = img.write_with_encoder(encoder).unwrap();
            let _result = buff.flush().unwrap();
            let slice = buff.into_inner().unwrap();
            slice
        }
        None => Vec::new()
    }
}

pub fn load_image_from_vec(vec: &Vec<u8>) -> Option<DynamicImage> {
    return match image::load_from_memory(vec) {
        Ok(dynamic_image) => Some(dynamic_image),
        Err(error) => {
            println!("error [processing_image] {}", error);
            None
        }
    };
}


fn process_jpg(path: &PathBuf, args: &Args) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let file_path = format!("{}{}", args.output, file_name);
    let re_jpg = Regex::new(r"\.jpeg$").unwrap();
    let img = image::open(&path);
    if img.is_ok() {
        let img = &img.unwrap();
        let new_file_path = re_jpg.replace_all(file_path.as_str(), ".jpg").to_string(); //file_path.replace(".jpeg", ".jpg");
        let current_aspect = Fraction::from(img.width()) / Fraction::from(img.height());
        let img = crop_image(&img, current_aspect, args.aspect_ratio);
        let img = resize_image(&img, args.max_width);
        let buff = File::create(&new_file_path).unwrap();
        let mut buff = BufWriter::new(buff);
        let encoder = JpegEncoder::new_with_quality(&mut buff, args.quality);
        let _result = img.write_with_encoder(encoder).unwrap();
        let _result = buff.flush().unwrap();
        copy_metadata(path.to_str().unwrap(), new_file_path.as_str())
    }
}


fn process_png(path: &PathBuf, args: &Args) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_file_path = format!("{}{}", args.output, file_name);
    let img = image::open(path);
    if img.is_ok() {
        let img = &img.unwrap();
        let current_aspect = Fraction::from(img.width()) / Fraction::from(img.height());
        let img = crop_image(&img, current_aspect, args.aspect_ratio);
        let img = resize_image(&img, args.max_width);
        let buff = File::create(&new_file_path).unwrap();
        let mut buff = BufWriter::new(buff);
        let encoder = PngEncoder::new_with_quality(&mut buff, image::codecs::png::CompressionType::Best, image::codecs::png::FilterType::Adaptive);
        let _result = img.write_with_encoder(encoder).unwrap();
        let _result = buff.flush().unwrap();
        copy_metadata(path.to_str().unwrap(), new_file_path.as_str())
    }
}

pub fn resize_image(img: &DynamicImage, max_width: u32) -> DynamicImage {
    let max_width = max_width as f64;
    let current_width = img.width() as f64;
    let current_height = img.height() as f64;

    let new_width = if current_width > max_width { max_width } else { current_width } as u32;
    let new_height = if current_width > max_width { (max_width / current_width) * current_height } else { current_height } as u32;

    img.resize_exact(new_width, new_height, image::imageops::FilterType::CatmullRom)
}

pub fn crop_image(img: &DynamicImage, current_aspect: Fraction, new_aspect: Fraction) -> DynamicImage {
    if new_aspect < current_aspect { // too wide
        // anchor on height, center on width
        let current_height = img.height();
        let current_width = img.width();
        let new_width = (current_height as f64 * new_aspect.to_f64().unwrap()) as u32;
        let x = ((current_width - new_width) as f64 / 2.0) as u32;
        img.crop_imm(x, 0, new_width, current_height)
    } else { // too narrow
        // anchor on width, center on height
        let current_height = img.height();
        let current_width = img.width();
        let new_height = (current_width as f64 / new_aspect.to_f64().unwrap()) as u32;
        let y = ((current_height - new_height) as f64 / 2.0) as u32;
        img.crop_imm(0, y, current_width, new_height)
    }
}

pub fn copy_metadata(source_path: &str, target_path: &str) {
    let meta = rexiv2::Metadata::new_from_path(source_path).unwrap();
    meta.clear_tag("Exif.Image.ImageLength");
    meta.clear_tag("Exif.Image.ImageWidth");
    let _result = meta.save_to_file(target_path);
}
