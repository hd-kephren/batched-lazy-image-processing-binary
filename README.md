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
  -f, --formats <FORMATS>            Picture formats by extension to process [default: gif|jpg|jpeg|png]
  -i, --input <INPUT>                Input directory for source images [default: ./input/]
  -m, --max-width <MAX_WIDTH>        Max width of image allowed before resizing [default: 1200]
  -o, --output <OUTPUT>              Output directory for processed images [default: ./output/]
  -q, --quality <QUALITY>            JPEG quality [default: 95]
  -h, --help                         Print help
  -V, --version                      Print version
```
**Will run with without any flags using the default directory and settings.*

### Example Run
```
Settings:
formats to import: gif|jpg|jpeg|png
batch size: 100
input directory: ./input/
output directory: ./output/
max image width: 1200

Processing 3519 files in 36 chunks.
███████████████████████████████████████████████████████████████████████████████ 3519/3519
Complete.
```

## TODOs
- animated GIF resizing
- more configuration as flags
- clean up after being more familiar with Rust
- file name normalization for SKUs