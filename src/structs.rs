use clap::Parser;
use fraction::Fraction;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Enforced aspect ratio with center crop
    #[arg(short, long, default_value = "5/7")]
    pub aspect_ratio: Fraction,

    /// Batch sizes of images to process in parallel
    #[arg(short, long, default_value = "100")]
    pub batch_size: usize,

    /// Picture formats by extension to process
    #[arg(short, long, default_value = "gif|jpg|jpeg|png")]
    pub extensions: String,

    /// Input directory for source images
    #[arg(short, long, default_value = "./input/")]
    pub input: String,

    /// Max width of image allowed before resizing.
    #[arg(short, long, default_value = "1500")]
    pub max_width: f64,

    /// Do not crop the image
    #[arg(long)]
    pub no_crop: bool,

    /// Do not copy EXIF/XMP/IPTC Metadata
    #[arg(long)]
    pub no_metadata: bool,

    /// Do not resize the image
    #[arg(long)]
    pub no_resize: bool,

    /// Output directory for processed images
    #[arg(short, long, default_value = "./output/")]
    pub output: String,

    /// JPEG quality
    #[arg(short, long, default_value = "95")]
    pub quality: u8,

    /// Initialize with UI
    #[arg(long)]
    pub ui: bool,
}