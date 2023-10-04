use std::ffi::OsStr;
use std::fs::{DirEntry, File};
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

use fraction::{Fraction, ToPrimitive};
use gif::Encoder;
use image::DynamicImage;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use regex::Regex;

use crate::structs::Args;

pub fn process_image(file: &std::io::Result<DirEntry>, args: Args) {
    let args = args.clone();
    let path = file.as_ref().unwrap().path();
    process_image_from_path(path, args);
}

pub fn process_image_from_path(path: PathBuf, args: Args) {
    let args = args.clone();
    let file_extension = path.extension().and_then(OsStr::to_str);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let _result = match file_extension {
        None => (),
        Some("jpg" | "jpeg") => process_jpg(path, args),
        Some("gif" | "png") => process_gif_png(path, args),
        Some(ext) => {
            println!("{} | Image format '{}' not supported.", file_name, ext)
        }
    };
}
pub fn process_in_memory_image(image: Option<DynamicImage>, args: Args) -> Vec<u8> {
    match image {
        Some(img) => {
            let re_jpg = Regex::new(r"\.jpeg$").unwrap();
            // let new_file_path = re_jpg.replace_all(file_path.as_str(), ".jpg").to_string(); //file_path.replace(".jpeg", ".jpg");
            let current_aspect = Fraction::from(img.width()) / Fraction::from(img.height());
            let img = if !args.no_crop { crop_jpg_png(img, current_aspect, args.aspect_ratio) } else { img };
            let img = if !args.no_crop { resize_jpg_png(img, args.max_width) } else { img };
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
    }
}


fn process_jpg(path: PathBuf, args: Args) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let file_path = format!("{}{}", args.output, file_name);
    let re_jpg = Regex::new(r"\.jpeg$").unwrap();
    let img = image::open(path.clone());
    if img.is_ok() {
        let img = img.unwrap();
        let new_file_path = re_jpg.replace_all(file_path.as_str(), ".jpg").to_string(); //file_path.replace(".jpeg", ".jpg");
        let current_aspect = Fraction::from(img.width()) / Fraction::from(img.height());
        let img = if !args.no_crop { crop_jpg_png(img, current_aspect, args.aspect_ratio) } else { img };
        let img = if !args.no_crop { resize_jpg_png(img, args.max_width) } else { img };
        let buff = File::create(new_file_path.clone()).unwrap();
        let mut buff = BufWriter::new(buff);
        let encoder = JpegEncoder::new_with_quality(&mut buff, args.quality);
        let _result = img.write_with_encoder(encoder).unwrap();
        let _result = buff.flush().unwrap();
        if !args.no_metadata { copy_metadata(path.to_str().unwrap(), new_file_path.as_str()) };
    }
}



fn process_gif_png(path: PathBuf, args: Args) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_file_path = format!("{}{}", args.output, file_name);
    let img = image::open(path.clone());
    if img.is_ok() {
        let img = img.unwrap();
        let current_aspect = Fraction::from(img.width()) / Fraction::from(img.height());
        let img = if !args.no_crop { crop_jpg_png(img, current_aspect, args.aspect_ratio) } else { img };
        let img = if !args.no_crop { resize_jpg_png(img, args.max_width) } else { img };
        let _result = img.save(new_file_path.clone());
        if !args.no_metadata { copy_metadata(path.to_str().unwrap(), new_file_path.as_str()) };
    }
}

fn process_animated_gif(path: PathBuf, args: Args) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_file_path = format!("{}{}", args.output, file_name);
    let decoder = gif::DecodeOptions::new();
    // Configure the decoder such that it will expand the image to RGBA.
    let file = File::open(path.clone()).unwrap();
    let decoder = decoder.read_info(file);
    if decoder.is_ok() {
        let mut decoder = decoder.unwrap();
        let mut image_file = File::create(new_file_path.clone()).unwrap();

        let mut encoder = Encoder::new(&mut image_file, decoder.width(), decoder.height(), decoder.global_palette().unwrap()).unwrap();
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            // Process every frame
            encoder.write_frame(&frame).unwrap();
        }
    }
    if !args.no_metadata { copy_metadata(path.to_str().unwrap(), new_file_path.as_str()) };
}

pub fn resize_jpg_png(img: DynamicImage, max_width: u32) -> DynamicImage {
    let max_width = max_width as f64;
    let current_width = img.width() as f64;
    let current_height = img.height() as f64;

    let resize = current_width > max_width;

    let new_width = if current_width > max_width { max_width } else { current_width } as u32;
    let new_height = if current_width > max_width { (max_width / current_width) * current_height } else { current_height } as u32;

    let img = if resize {
        return img.resize_exact(new_width, new_height, FilterType::CatmullRom);
    } else {
        img
    };
    return img;
}

pub fn crop_jpg_png(img: DynamicImage, current_aspect: Fraction, new_aspect: Fraction) -> DynamicImage {
    if new_aspect < current_aspect { // too wide
        // anchor on height, center on width
        let current_height = img.height();
        let current_width = img.width();
        let new_width = (current_height as f64 * new_aspect.to_f64().unwrap()) as u32;
        let x = ((current_width - new_width) as f64 / 2.0) as u32;
        img.crop_imm(x, 0, new_width, current_height)
    } else if new_aspect > current_aspect { // too narrow
        // anchor on width, center on height
        let current_height = img.height();
        let current_width = img.width();
        let new_height = (current_width as f64 / new_aspect.to_f64().unwrap()) as u32;
        let y = ((current_height - new_height) as f64 / 2.0) as u32;
        img.crop_imm(0, y, current_width, new_height)
    } else { // just right, noop
        img
    }
}

pub fn copy_metadata(source_path: &str, target_path: &str) {
    // let meta = rexiv2::Metadata::new_from_path(source_path).unwrap();
    // meta.clear_tag("Exif.Image.ImageLength");
    // meta.clear_tag("Exif.Image.ImageWidth");
    // let _result = meta.save_to_file(target_path);
}
