use std::fs;
use clap::Parser;
use indicatif::ProgressBar;
use rayon::prelude::*;

use crate::imports::directory_to_files;
use crate::process::process_image;
use crate::structs::Args;

mod imports;
mod process;
mod ui;
mod structs;


fn main() {
    let args = Args::parse();
    let _result = fs::create_dir_all(&args.input);
    let _result = fs::create_dir_all(&args.output);
    rexiv2::initialize().expect("Unable to initialize 'rexiv2'. Please check the readme.md for external requirements.");
    if args.ui {
        ui::run(args);
    } else {
        let batch_size = args.batch_size;
        let input = args.input.as_str();
        println!(":::::Settings:::::\naspect ratio: {}\nimage type to decode: {}\nbatch size: {}\ninput directory: {}\noutput directory: {}\nmax image width: {}\nJPEG quality: {}\n", args.aspect_ratio, args.decode, args.batch_size, args.input, args.output, args.max_width, args.quality);

        let extensions: Vec<&str> = args.decode.split("|").collect();
        let filtered_files = directory_to_files(&input, &extensions);
        let count = filtered_files.iter().count();
        let chunks = (count as f64 / batch_size as f64).ceil();
        println!("Processing {} files in {} chunks.", count, chunks);

        let progress_bar = ProgressBar::new(count as u64);
        filtered_files
            .chunks(batch_size)
            .for_each(|filtered_files_of_files| {
                filtered_files_of_files
                    .par_iter()
                    .for_each(|file| {
                        progress_bar.inc(1);
                        process_image(file, &args);
                    })
            });
        progress_bar.finish();
        println!("\nComplete.");
    }
}



