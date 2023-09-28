use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use fraction::{Fraction, ToPrimitive};
use gif::Encoder;
use indicatif::ProgressBar;
use image::{DynamicImage, GenericImageView};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use rayon::prelude::*;
use regex::Regex;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enforced aspect ratio with center crop
    #[arg(short, long, default_value = "5/7")]
    aspect_ratio: Fraction,

    /// Batch sizes of images to process in parallel
    #[arg(short, long, default_value = "100")]
    batch_size: usize,

    /// Picture formats by extension to process
    #[arg(short, long, default_value = "gif|jpg|jpeg|png")]
    extensions: String,

    /// Input directory for source images
    #[arg(short, long, default_value = "./input/")]
    input: String,

    /// Max width of image allowed before resizing.
    #[arg(short, long, default_value = "1500")]
    max_width: f64,

    /// Do not crop the image
    #[arg(long)]
    no_crop: bool,

    /// Do not copy EXIF/XMP/IPTC Metadata
    #[arg(long)]
    no_metadata: bool,

    /// Do not resize the image
    #[arg(long)]
    no_resize: bool,

    /// Output directory for processed images
    #[arg(short, long, default_value = "./output/")]
    output: String,

    /// JPEG quality
    #[arg(short, long, default_value = "95")]
    quality: u8,
}

fn main() {
    let args = Args::parse();
    let batch_size = args.batch_size;
    let input = args.input.clone();
    println!(":::::Settings:::::\naspect ratio: {}\nextensions to process: {}\nbatch size: {}\ninput directory: {}\noutput directory: {}\nmax image width: {}\nskip cropping: {}\nskip metadata: {}\nskip resizing: {}\nJPEG quality: {}\n", args.aspect_ratio, args.extensions, args.batch_size, args.input, args.output, args.max_width, args.no_crop, args.no_metadata, args.no_resize, args.quality);
    rexiv2::initialize().expect("Unable to initialize 'rexiv2'. Please check the readme.md for external requirements.");

    let paths = fs::read_dir(input).unwrap();
    let extensions: Vec<&str> = args.extensions.split("|").collect();
    let filtered_files: Vec<_> = paths
        .into_iter()
        .filter(|path| {
            let file_name = path.as_ref().unwrap().file_name().into_string().unwrap();
            let file_extension = file_name.split(".").last().unwrap();
            return extensions.contains(&file_extension);
        })
        .collect();
    let count = filtered_files.iter().count();
    let chunks = (count as f64 / batch_size as f64).ceil();
    println!("Processing {} files in {} chunks.", count, chunks);
    let re_jpg = Regex::new(r"\.jpeg$").unwrap();
    let progress_bar = ProgressBar::new(count as u64);
    filtered_files
        .chunks(batch_size)
        .for_each(|filtered_files_of_files| {
            filtered_files_of_files
                .par_iter()
                .for_each(|file| {
                    progress_bar.inc(1);
                    let args = args.clone();
                    let path = file.as_ref().unwrap().path();
                    let file_extension = path.extension().and_then(OsStr::to_str);
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let _result = match file_extension {
                        None => (),
                        Some("jpg" | "jpeg") => process_jpg(path, args, &re_jpg),
                        Some("gif" | "png") => process_gif_png(path, args),
                        Some(ext) => {
                            println!("{} | Image format '{}' not supported.", file_name, ext)
                        }
                    };
                })
        });
    progress_bar.finish();
    println!("\nComplete.");
}

fn process_jpg(path: PathBuf, args: Args, re_jpg: &Regex) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let file_path = format!("{}{}", args.output, file_name);

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
        if !args.no_metadata {copy_metadata(path.to_str().unwrap(), new_file_path.as_str())};
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
        if !args.no_metadata {copy_metadata(path.to_str().unwrap(), new_file_path.as_str())};
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
    if !args.no_metadata {copy_metadata(path.to_str().unwrap(), new_file_path.as_str())};
}

fn resize_jpg_png(img: DynamicImage, max_width: f64) -> DynamicImage {
    let current_width = img.dimensions().0 as f64;
    let current_height = img.dimensions().1 as f64;

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

fn crop_jpg_png(img: DynamicImage, current_aspect: Fraction, new_aspect: Fraction) -> DynamicImage {
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

fn copy_metadata(source_path: &str, target_path: &str) {
    let meta = rexiv2::Metadata::new_from_path(source_path).unwrap();
    meta.clear_tag("Exif.Image.ImageLength");
    meta.clear_tag("Exif.Image.ImageWidth");
    let _result = meta.save_to_file(target_path);
}