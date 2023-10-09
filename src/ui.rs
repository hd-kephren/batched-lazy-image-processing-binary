use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use atomic_float::AtomicF32;

use eframe::{egui, Renderer};
use eframe::egui::{Align, ColorImage, ImageData, Slider, SliderOrientation, TextureHandle, TextureOptions};
use fraction::Fraction;
use image::{DynamicImage, EncodableLayout};
use regex::Regex;

use crate::imports::directory_to_files;
use crate::process::{load_image_from_vec, process_images, process_image_in_memory};
use crate::structs::Args;

pub fn run(settings: Args) {
    let native_options = eframe::NativeOptions {
        renderer: Renderer::Wgpu,
        ..Default::default()
    };
    let _ = eframe::run_native("Batched Lazy Image Processing Binary", native_options, Box::new(|cc| Box::new(App::new(cc, settings))));
}

static PROGRESS: AtomicF32 = AtomicF32::new(0.0);

struct App {
    jpeg_quality: u32,
    target_max_width: u32,
    source_max_width: u32,
    source_min_width: u32,
    aspect_ratio: String,
    batch_size: usize,
    decode: String,
    encode: String,
    preview: bool,
    input: String,
    output: String,
    files: Vec<std::io::Result<DirEntry>>,
    file_count: usize,
    file_selected: usize,
    source_file_name: Option<String>,
    existing_extension: String,
    source_path: Option<PathBuf>,
    source_image: Option<DynamicImage>,
    source_texture: Option<TextureHandle>,
    target_texture: Option<TextureHandle>,
    update: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, settings: Args) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        egui_extras::install_image_loaders(&cc.egui_ctx);
        let input_directory = settings.input.as_str();
        let extensions: Vec<&str> = settings.decode.split("|").collect();
        let files = directory_to_files(input_directory, &extensions);
        let file_count = files.iter().count();
        let mut existing_extension = String::from("");
        let file_name_and_path = if files.iter().count() > 0 {
            let file = files.get(0).unwrap();
            let path = file.as_ref().map(|f| { f.path() }).unwrap();
            let e = path.extension();
            existing_extension = String::from(e.and_then(OsStr::to_str).unwrap());
            let file_name = path.file_name().map(|s| s.to_os_string().into_string().unwrap());
            let source_image = match image::open(&path) {
                Ok(image) => Some(image),
                Err(_) => None
            };
            (file_name, Some(path), source_image)
        } else {
            (None, None, None)
        };
        let (source_file_name, source_path, source_image) = file_name_and_path;
        App {
            jpeg_quality: (settings.quality as u32),
            target_max_width: settings.max_width,
            source_max_width: 0u32,
            source_min_width: 0u32,
            aspect_ratio: settings.aspect_ratio.to_string(),
            batch_size: settings.batch_size,
            decode: settings.decode.clone(),
            encode: settings.encode.clone(),
            existing_extension,
            preview: false,
            input: settings.input.clone(),
            output: settings.output,
            files,
            file_count,
            file_selected: 1,
            source_path,
            source_file_name,
            source_image,
            source_texture: None,
            target_texture: None,
            update: true,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let frame_size = frame.info().window_info.size;
            let half_frame_width = frame_size.x / 2.0;
            let slider_spacing = half_frame_width - 43.0;
            ui.style_mut().spacing.slider_width = slider_spacing;
            ui.vertical_centered(|ui| {
                egui::TopBottomPanel::top("top_panel")
                    .resizable(false)
                    .min_height(100.0)
                    .show_inside(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.set_height(20.0);
                            let mut size = ui.available_size();
                            size.x = half_frame_width;
                            let text = egui::TextEdit::singleline(&mut self.input)
                                .horizontal_align(Align::Center);
                            ui.add_sized(size, text);
                            if ui.button("Input folder...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_directory(&self.input)
                                    .pick_folder() {
                                    let extensions = self.decode.split("|").collect();
                                    let re = Regex::new(r"/+$").unwrap();
                                    let input = path.display().to_string() + "/";
                                    let input_with_slash = re.replace_all(input.as_str(), "/");
                                    self.input = input_with_slash.to_string();
                                    self.files = directory_to_files(path.display().to_string().as_str(), &extensions);
                                    self.file_count = self.files.iter().count();
                                    self.file_selected = 1;
                                    let file = self.files.get(self.file_selected - 1).unwrap();
                                    let path = file.as_ref().map(|f| { f.path() }).unwrap();
                                    let extension = path.extension();
                                    let existing_extension = String::from(extension.and_then(OsStr::to_str).unwrap());
                                    let file_name = path.file_name().map(|s| s.to_os_string().into_string().unwrap());
                                    self.source_file_name = file_name;
                                    self.source_path = Some(path.clone());
                                    self.preview = false;
                                    self.source_image = match image::open(path) {
                                        Ok(image) => Some(image),
                                        Err(_) => None
                                    };
                                    self.update = true;
                                    self.existing_extension = existing_extension;
                                }

                            }
                            if ui.button("Refresh").clicked() {
                                let extensions = self.decode.split("|").collect();
                                self.files = directory_to_files(self.input.as_str(), &extensions);
                                self.file_count = self.files.iter().count();
                                self.file_selected = 1;
                                let file = self.files.get(self.file_selected - 1).unwrap();
                                let path = file.as_ref().map(|f| { f.path() }).unwrap();
                                let extension = path.extension();
                                let existing_extension = String::from(extension.and_then(OsStr::to_str).unwrap());
                                let file_name = path.file_name().map(|s| s.to_os_string().into_string().unwrap());
                                self.source_file_name = file_name;
                                self.source_path = Some(path.clone());
                                self.source_image = match image::open(path) {
                                    Ok(image) => Some(image),
                                    Err(_) => {
                                        self.preview = false;
                                        None
                                    }
                                };
                                self.update = true;
                                self.existing_extension = existing_extension;
                            }
                        });
                        ui.separator();
                        ui.horizontal_top(|ui| {
                            ui.set_height(20.0);
                            let mut size = ui.available_size();
                            size.x = half_frame_width;
                            let text = egui::TextEdit::singleline(&mut self.output)
                                .horizontal_align(Align::Center);
                            ui.add_sized(size, text);
                            if ui.button("Output folder...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_directory(&self.output)
                                    .pick_folder() {
                                    let re = Regex::new(r"/+$").unwrap();
                                    let output = path.display().to_string() + "/";
                                    let output_with_slash = re.replace_all(output.as_str(), "/");
                                    self.output = output_with_slash.to_string();
                                }
                                self.update = true;
                            }
                        });
                        ui.separator();
                        ui.horizontal_top(|ui| {
                            ui.set_height(20.0);
                            let mut size = ui.available_size();
                            size.x = half_frame_width;
                            let text = egui::TextEdit::singleline(&mut self.aspect_ratio)
                                .horizontal_align(Align::Center);
                            if ui.add_sized(size, text).changed() {
                                self.update = true;
                            };
                            ui.label("Aspect Ratio");
                        });
                        ui.separator();
                        if ui.add(Slider::new(&mut self.target_max_width, self.source_min_width..=self.source_max_width)
                            .orientation(SliderOrientation::Horizontal)
                            .text("Maximum Width")
                            .trailing_fill(false)
                        ).changed() {
                            self.update = true;
                        };
                        ui.separator();
                        if ui.add(Slider::new(&mut self.jpeg_quality, 0u32..=100u32)
                            .clamp_to_range(true)
                            .smart_aim(true)
                            .trailing_fill(true)
                            .orientation(SliderOrientation::Horizontal)
                            .text("JPEG Quality")
                        ).changed() {
                            self.update = true;
                        };
                        ui.add_space(5.0);
                        ui.horizontal_top(|ui| {
                            let slider = Slider::new(&mut self.file_selected, 1usize..=self.file_count)
                                .clamp_to_range(true)
                                .smart_aim(true)
                                .trailing_fill(false)
                                .orientation(SliderOrientation::Horizontal)
                                .text(format!(" of {} Files", self.file_count));
                            if ui.add(slider).changed() {
                                let file = self.files.get(self.file_selected - 1).unwrap();
                                let path = file.as_ref().map(|f| { f.path() }).unwrap();
                                let file_name = path.file_name().map(|s| s.to_os_string().into_string().unwrap());
                                self.source_file_name = file_name;
                                self.source_path = Some(path);
                                if self.source_file_name.is_some() && self.source_path.is_some() {
                                    self.source_path.iter().for_each(|path| {
                                        self.source_image = match image::open(path) {
                                            Ok(image) => Some(image),
                                            Err(_) => None
                                        };
                                    });
                                };
                                self.update = true;
                            };
                            ui.add_space(5.0);
                            ui.vertical(|ui| {
                                if ui.checkbox(&mut self.preview, "Live Preview").changed() {
                                    self.update = true;
                                };
                            });
                        });
                        ui.add_space(5.0);
                    });
                ui.add_space(5.0);
                ui.horizontal_top(|ui| {
                    let button = egui::Button::new("Process Images");
                    if PROGRESS.load(Ordering::SeqCst) == 0.0
                        || PROGRESS.load(Ordering::SeqCst) == 1.0 {
                        if ui.add(button).clicked() {
                            let args = build_args_from_app(self);
                            PROGRESS.swap(0.0, Ordering::SeqCst);
                            thread::spawn(move || {
                                process_images(&args, &PROGRESS);
                            });
                        }
                    } else {
                        ui.add_enabled(false, button);
                    }

                    ui.add(egui::ProgressBar::new(PROGRESS.load(Ordering::SeqCst)).show_percentage());
                    if PROGRESS.load(Ordering::SeqCst) < 1.0 && PROGRESS.load(Ordering::SeqCst) > 0.0 {
                        ctx.request_repaint_after(Duration::from_secs(1));
                    }
                });
                ui.separator();
                ui.columns(2, |cols| {
                    for (i, col) in cols.iter_mut().enumerate() {
                        if i == 0 {
                            col.vertical(|col| {
                                col.label(format!("Source Image: {}", self.source_file_name.as_ref().unwrap_or(&String::from("<None>"))));
                                if self.source_file_name.is_some() && self.source_path.is_some() {
                                    if self.update {
                                        self.source_texture = build_image_texture("source", &self.source_image, col);
                                        self.source_max_width = self.source_image.as_ref().map(|image| image.width()).unwrap_or(2048u32);
                                        self.source_min_width = if self.source_max_width < 32 { self.source_max_width / 2u32 } else { 32u32 };
                                    };

                                    match &self.source_texture {
                                        Some(handle) => {
                                            egui::ScrollArea::both().show(col, |col| {
                                                col.image((handle.id(), handle.size_vec2()));
                                            });
                                        }
                                        None => ()
                                    }
                                };
                            });
                        } else {
                            if self.preview && self.update {
                                let args = build_args_from_app(self);
                                if self.source_image.is_some() {
                                    let buffer = process_image_in_memory(&self.source_image, &args, self.existing_extension.as_str());
                                    let target_image = &load_image_from_vec(&buffer);
                                    self.target_texture = build_image_texture("target", target_image, col);
                                };
                            }
                            col.vertical(|col| {
                                col.label(format!("Target Image: {}", self.source_file_name.as_ref().unwrap_or(&String::from("<None>"))));
                                if self.preview {
                                    self.target_texture.as_ref().map(|target_handle| {
                                        egui::ScrollArea::both().show(col, |col| {
                                            col.image((target_handle.id(), target_handle.size_vec2()));
                                        });
                                    });
                                }
                            });
                        }
                    }
                });
            });
        });
        self.update = false;
    }
}

fn build_image_texture(name: &str, optional_image: &Option<DynamicImage>, ui: &mut egui::Ui) -> Option<TextureHandle> {
    optional_image.as_ref().map(|image| {
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_bytes();
        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels);
        ui.ctx().load_texture(name, ImageData::Color(Arc::new(color_image)), TextureOptions::default())
    })
}

fn build_args_from_app(app: &mut App) -> Args {
    let aspect_ratio = match Fraction::from_str(app.aspect_ratio.clone().as_str()) {
        Ok(ar) => ar,
        Err(error) => {
            println!("error [build_args_from_app] {}, returning an aspect ratio of 1.", error);
            Fraction::from(1)
        }
    };
    Args {
        aspect_ratio,
        batch_size: app.batch_size,
        decode: app.decode.clone(),
        encode: app.encode.clone(),
        input: app.input.clone(),
        max_width: app.target_max_width,
        output: app.output.clone(),
        quality: app.jpeg_quality as u8,
        ui: true,
    }
}