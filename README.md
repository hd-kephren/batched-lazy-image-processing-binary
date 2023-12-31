# batched-lazy-image-processing-binary
Batch image cropping and resizing tool built in Rust

## Compile ##
#### Cargo/CLion/RustRover #### 
**debug**: `cargo build --package batched-lazy-image-processing-binary --bin blipb`  
**release(optimized)**: `cargo build --package batched-lazy-image-processing-binary --bin blipb --release`  
-> `./target/<debug|release>/`

## Usage ##
`./blipb --help`

```
A batch image processor for cropping, reformatting, and resizing multiple images.

Usage: blipb [OPTIONS]

Options:
  -a, --aspect-ratio <ASPECT_RATIO>  Enforced aspect ratio with center crop [default: 5/7]
  -b, --batch-size <BATCH_SIZE>      Batch sizes of images to process in parallel [default: 100]
  -e, --extensions <EXTENSIONS>      Picture formats by extension to process [default: jpg|jpeg|png]
  -i, --input <INPUT>                Input directory for source images [default: ./input/]
  -m, --max-width <MAX_WIDTH>        Max width of image allowed before resizing [default: 1500]
  -o, --output <OUTPUT>              Output directory for processed images [default: ./output/]
  -q, --quality <QUALITY>            JPEG quality [default: 90]
      --ui                           Initialize with UI (still under major development)
  -h, --help                         Print help
  -V, --version                      Print version
```
**Will run with without any flags using the default directory and settings.*

### Example Run
`/blipb --max-width 1000 --quality 80`
```
:::::Settings:::::
extensions to process: jpg|jpeg|png
batch size: 100
input directory: ./input/
output directory: ./output/
max image width: 1000
JPEG quality: 80

Processing 3519 files in 36 chunks.
████████████████████████████████████████████████████████████████████████████████████ 3519/3519
Complete.
```

### Image Metadata and External Dependencies
- Metadata functionality is still experimental 
- Uses the library [**rexiv2**](https://github.com/felixc/rexiv2) to copy Metadata for images  
  This is a Rust wrapper for the [**gexiv2**](https://wiki.gnome.org/Projects/gexiv2) library, which is a wrapper around [exiv2](https://exiv2.org/)

## TODOs ##
- clean up after being more familiar with Rust
- file name normalization for SKUs
- continue UI cleanup and refinement

## UI ##
- [slint](https://slint.rs/)