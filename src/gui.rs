#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::gui::emath::vec2;
use crate::gui::egui::Window;
use crate::gui::epaint::Color32;
use eframe::egui;
use eframe::*;
use egui::containers::Frame;

pub fn gui() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(emath::Vec2{x:800.0,y:800.0});
    eframe::run_native(
        "Xadreis",
        options,
        Box::new(|_cc| Box::new(XadreisGUI::default())),
    );
}

struct XadreisGUI {

}

impl Default for XadreisGUI {
    fn default() -> Self {
        Self { }
    }
}

// TODO:
// - how to make the window only resizable diagonally?

impl eframe::App for XadreisGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     Frame::canvas(ui.style()).show(ui, |ui| {
        //         ui.ctx().request_repaint();

        //         let mut shapes = vec![];

        //         let size = _frame.info().window_info.size.x.min(_frame.info().window_info.size.y);

        //         for i in 0..8 {
        //             for j in 0..8 {
        //                 let tile_size = size/8.0;//80.0;
        //                 let x_pos = i as f32 * tile_size;
        //                 let y_pos = j as f32 * tile_size;
        //                 let color = if (i + j) % 2 == 0 {
        //                     Color32::from_rgb(210, 210, 155)
        //                 } else {
        //                     Color32::from_rgb(100, 55, 40)
        //                 };
        //                 shapes.push(epaint::Shape::Rect(epaint::RectShape::filled(
        //                     egui::Rect { min: egui::pos2(x_pos, y_pos),
        //                                  max: egui::pos2(x_pos + tile_size, y_pos + tile_size) },
        //                     epaint::Rounding::none(), color
        //                 )));
        //             }
        //         }

        //         ui.painter().extend(shapes);
        //     });
        // });

        let mut open = true;
        Window::new("Board")
            .open(&mut open)
            .default_size(vec2(512.0, 512.0))
            .vscroll(false)
            .show(ctx, |ui| {
                Frame::canvas(ui.style()).show(ui, |ui| {
                    ui.ctx().request_repaint();

                    let mut shapes = vec![];

                    //let size = _frame.info().window_info.size.x.min(_frame.info().window_info.size.y);
                    let size = ui.available_width();
                    let (_id, rect) = ui.allocate_space(size * vec2(1.0, 1.0));

                    for i in 0..8 {
                        for j in 0..8 {
                            let tile_size = rect.width()/8.0;//80.0;
                            let x_pos = i as f32 * tile_size + rect.min.x;
                            let y_pos = j as f32 * tile_size + rect.min.y;
                            let color = if (i + j) % 2 == 0 {
                                Color32::from_rgb(210, 210, 155)
                            } else {
                                Color32::from_rgb(100, 55, 40)
                            };
                            shapes.push(
                                epaint::Shape::Rect(epaint::RectShape::filled(
                                    egui::Rect {
                                        min: egui::pos2(x_pos, y_pos),
                                        max: egui::pos2(x_pos + tile_size,
                                                        y_pos + tile_size) },
                                epaint::Rounding::none(),
                                color
                            )));
                        }
                    }

                    ui.painter().extend(shapes);
                });
            });
    }
}

