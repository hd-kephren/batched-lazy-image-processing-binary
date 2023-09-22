# batched-lazy-image-processing-binary
Image cropping and resizing tool built in Rust

## Compile ##
#### Cargo/Intellij/RustRover #### 
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
  -i, --input <INPUT>                Input directory for source images [default: ./input/]
  -o, --output <OUTPUT>              Output directory for processed images [default: ./output/]
  -m, --max-width <MAX_WIDTH>        Max width of image allowed before resizing [default: 1200]
  -f, --formats <FORMATS>            Picture formats by extension to process [default: gif|jpg|jpeg|png]
  -h, --help                         Print help
  -V, --version                      Print version
```
**Will run with without any flags using the default directory and settings.*

## TODOs
- retain/append metadata
- file name normalization for SKUs
- animated GIFs
- more configuration as flags
- clean up after more familiarity with Rust