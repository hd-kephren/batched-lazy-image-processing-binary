use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use fraction::{Fraction, ToPrimitive};
use gif::Encoder;
use indicatif::ProgressIterator;
use image::{DynamicImage, GenericImageView};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use rayon::prelude::*;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enforced aspect ratio with center crop
    #[arg(short, long, default_value = "5/7")]
    aspect_ratio: Fraction,

    /// Batch sizes of images to process in parallel
    #[arg(short, long, default_value = "100")]
    batch_size: usize,

    /// Input directory for source images
    #[arg(short, long, default_value = "./input/")]
    input: String,

    /// Output directory for processed images
    #[arg(short, long, default_value = "./output/")]
    output: String,

    /// Max width of image allowed before resizing.
    #[arg(short, long, default_value = "1200")]
    max_width: f64,

    /// Picture formats by extension to process
    #[arg(short, long, default_value = "gif|jpg|jpeg|png")]
    formats: String,
}

fn main() {
    let args = Args::parse();
    let batch_size = args.batch_size;
    let input = args.input.clone();
    println!("Settings:\nformats to import: {}\nbatch size: {}\ninput directory: {}\noutput directory: {}\nmax image width: {}\n", args.formats, args.batch_size, args.input, args.output, args.max_width);
    let paths = fs::read_dir(input).unwrap();
    let formats: Vec<&str> = args.formats.split("|").collect();
    let filtered_files: Vec<_> = paths
        .into_iter()
        .filter(|path| {
            let file_name = path.as_ref().unwrap().file_name().into_string().unwrap();
            let file_extension = file_name.split(".").last().unwrap();
            return formats.contains(&file_extension);
        })
        .collect();
    let count = filtered_files.iter().count();
    let chunks = (count as f64 / batch_size as f64).ceil();
    println!("Processing {} files in {} chunks.", count, chunks);
    filtered_files
        .chunks(batch_size)
        .progress()
        .for_each(|filtered_files_of_files| {
            filtered_files_of_files
                .par_iter()
                .for_each(|file| {
                    let args = args.clone();
                    let path = file.as_ref().unwrap().path();
                    let file_extension = path.extension().and_then(OsStr::to_str);
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    // println!("loading image: {}", file_name);

                    let _result = match file_extension {
                        None => (),
                        Some("jpg" | "jpeg") => process_jpg(path, args),
                        Some("png") => process_png(path, args),
                        Some("gif") => process_gif(path, args),
                        Some(ext) => {
                            println!("{} | Image format '{}' not supported.", file_name, ext)
                        }
                    };
                })
        });
    println!("Complete.");
}

fn process_jpg(path: PathBuf, args: Args) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_file_path = format!("{}{}", args.output, file_name);
    let img = image::open(path.clone());
    if img.is_ok() {
        let img = img.unwrap();
        let current_aspect = Fraction::from(img.width()) / Fraction::from(img.height());
        let img = crop_jpg_png(img, current_aspect, args.aspect_ratio);
        let img = resize_jpg_png(img, args.max_width);
        let buff = File::create(new_file_path.clone()).unwrap();
        let mut buff = BufWriter::new(buff);
        let encoder = JpegEncoder::new_with_quality(&mut buff, 95);
        let _result = img.write_with_encoder(encoder).unwrap();
        let _result = buff.flush().unwrap();
        copy_metadata(path.to_str().unwrap(), new_file_path.as_str());
    }
}

fn process_png(path: PathBuf, args: Args) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_file_path = format!("{}{}", args.output, file_name);

    let img = image::open(path.clone());
    if img.is_ok() {
        let img = resize_jpg_png(img.unwrap(), args.max_width);
        // println!("saving image: {}", file_name);
        let _result = img.save(new_file_path.clone());
        let source_path = path.to_str().unwrap();
        copy_metadata_with_exiftool(source_path, new_file_path.as_str());
    }
}

fn process_gif(path: PathBuf, args: Args) {
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
    let source_path = path.to_str().unwrap();
    copy_metadata_with_exiftool(source_path, new_file_path.as_str());
}

fn resize_jpg_png(img: DynamicImage, max_width: f64) -> DynamicImage {
    let current_width = img.dimensions().0 as f64;
    let current_height = img.dimensions().1 as f64;

    let resize = current_width > max_width;

    let new_width = if current_width > max_width { max_width } else { current_width } as u32;
    let new_height = if current_width > max_width { (max_width / current_width) * current_height } else { current_height } as u32;

    let img = if resize {
        // println!("resizing {:?} -> ({},{})", img.dimensions(), new_width, new_height);
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

fn copy_metadata_with_exiftool(source_path: &str, target_path: &str) {
    //exiftool -TagsFromFile srcimage.jpg "-all:all>all:all" targetimage.jpg
    let arg = format!("exiftool -overwrite_original -TagsFromFile \"{}\" \"-all:all>all:all\" \"{}\"", source_path, target_path);
    let error_message = "failed to execute Metadata copy process";
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &arg])
            .output()
            .expect(error_message)
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(arg)
            .output()
            .expect(error_message)
    };
    let _hello = output.stdout;
}

fn copy_metadata(source_path: &str, target_path: &str) {
    let meta = rexiv2::Metadata::new_from_path(source_path).unwrap();
    meta.clear_tag("Exif.Image.ImageLength");
    meta.clear_tag("Exif.Image.ImageWidth");
    let _result = meta.save_to_file(target_path);
}