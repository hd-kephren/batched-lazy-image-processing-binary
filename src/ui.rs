use eframe::egui;
use eframe::egui::{Slider, SliderOrientation};

pub fn run() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Batched Lazy Image Processing Binary", native_options, Box::new(|cc| Box::new(App::new(cc))));
}

#[derive(Default)]
struct App {
    jpeg_quality: u32,
    maximum_width: u32
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let this = App {
            jpeg_quality:95u32,
            maximum_width: 1500,
            ..Default::default()
        };
        this
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let frame_size = frame.info().window_info.size;
            let half_frame_width = frame_size.x / 2.0;
            ui.style_mut().spacing.slider_width = (frame_size.x / 2.0);
            ui.vertical(|ui| {
                egui::TopBottomPanel::top("top_panel")
                    .resizable(true)
                    .min_height(100.0)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("Settings");
                            });
                            ui.add(
                                Slider::new(&mut self.jpeg_quality, 0u32..=100u32)
                                    .clamp_to_range(true)
                                    .smart_aim(true)
                                    .trailing_fill(true)
                                    .orientation(SliderOrientation::Horizontal)
                                    .text("JPEG Quality")
                            );
                            ui.separator();
                            ui.add(
                                Slider::new(&mut self.maximum_width, 1000u32..=2048u32)
                                    .orientation(SliderOrientation::Horizontal)
                                    .text("Maximum Width")
                                    .trailing_fill(false)
                            );
                            ui.separator();
                        });
                    });
                // egui::CentralPanel::default()
                //     .show_inside(ui, |ui| {
                //         ui.vertical(|ui| {
                //             ui.heading("Input Image");
                //         });
                //         egui::ScrollArea::vertical().show(ui, |ui| {
                //             // lorem_ipsum(ui);
                //         });
                //     });
                egui::SidePanel::right("right_panel")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.heading("Output Image");
                        });
                        egui::ScrollArea::vertical().show(ui, |ui| {
//                            lorem_ipsum(ui);
                        });
                    });
            });

            ui.vertical(
                |ui|{
                    ui.horizontal(|ui| {

                        ui.separator();
                        ui.add(
                            Slider::new(&mut 1500, 1000u32..=2048u32)
                                .orientation(SliderOrientation::Horizontal)
                                .text("Maximum Width")
                        )
                    });
                }
            );
        });
    }
}