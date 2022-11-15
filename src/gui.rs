#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::fen_to_game_state;
use crate::PieceType;
use crate::GameState;
use crate::Player;
use egui_extras::RetainedImage;
use crate::gui::emath::vec2;
use crate::gui::egui::Window;
use crate::gui::epaint::Color32;
use eframe::egui;
use eframe::*;
use egui::containers::Frame;

pub fn gui(game_state: GameState, fen: String) {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(emath::Vec2{x:800.0,y:800.0});
    eframe::run_native(
        "Xadreis",
        options,
        Box::new(|_cc| Box::new(XadreisGUI::from_game_state(game_state, fen))),
    );
}

struct XadreisGUI {
    wbishop: RetainedImage,
    wrook: RetainedImage,
    wking: RetainedImage,
    wqueen: RetainedImage,
    wknight: RetainedImage,
    wpawn: RetainedImage,

    bbishop: RetainedImage,
    brook: RetainedImage,
    bking: RetainedImage,
    bqueen: RetainedImage,
    bknight: RetainedImage,
    bpawn: RetainedImage,

    game_state: Option<GameState>,
    fen: String,
}

impl XadreisGUI {
    fn from_game_state(game_state: GameState, fen: String) -> Self {
        let mut s = Self::default();
        s.game_state = Some(game_state);
        s.fen = fen;

        return s;
    }
}

impl Default for XadreisGUI {
    fn default() -> Self {
        Self {
            wbishop: RetainedImage::from_image_bytes("wbishop", include_bytes!("../assets/pieces/wbishop.png")).unwrap(),
            wrook: RetainedImage::from_image_bytes("wrook", include_bytes!("../assets/pieces/wrook.png")).unwrap(),
            wking: RetainedImage::from_image_bytes("wking", include_bytes!("../assets/pieces/wking.png")).unwrap(),
            wqueen: RetainedImage::from_image_bytes("wqueen", include_bytes!("../assets/pieces/wqueen.png")).unwrap(),
            wknight: RetainedImage::from_image_bytes("wknight", include_bytes!("../assets/pieces/wknight.png")).unwrap(),
            wpawn: RetainedImage::from_image_bytes("wpawn", include_bytes!("../assets/pieces/wpawn.png")).unwrap(),

            bbishop: RetainedImage::from_image_bytes("bbishop", include_bytes!("../assets/pieces/bishop.png")).unwrap(),
            brook: RetainedImage::from_image_bytes("brook", include_bytes!("../assets/pieces/rook.png")).unwrap(),
            bking: RetainedImage::from_image_bytes("bking", include_bytes!("../assets/pieces/king.png")).unwrap(),
            bqueen: RetainedImage::from_image_bytes("bqueen", include_bytes!("../assets/pieces/queen.png")).unwrap(),
            bknight: RetainedImage::from_image_bytes("bknight", include_bytes!("../assets/pieces/knight.png")).unwrap(),
            bpawn: RetainedImage::from_image_bytes("bpawn", include_bytes!("../assets/pieces/pawn.png")).unwrap(),
        
            game_state: None,
            fen: "".to_string(),
        }
    }
}

impl eframe::App for XadreisGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut open = true;
        Window::new("Board")
            .open(&mut open)
            .default_size(vec2(512.0, 512.0))
            .vscroll(false)
            .show(ctx, |ui| {
                let response = ui.add(egui::TextEdit::singleline(&mut self.fen).desired_width(f32::INFINITY).hint_text("Paste FEN here..."));
                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    self.fen = self.fen.trim().to_string();
                    self.game_state = Some(fen_to_game_state(self.fen.to_string()));
                }
                Frame::canvas(ui.style()).show(ui, |ui| {
                    ui.ctx().request_repaint();

                    let size = ui.available_width();
                    let (_id, board_rect) = ui.allocate_space(size * vec2(1.0, 1.0));

                    let mut shapes = vec![];

                    for i in 0..8 {
                        for j in 0..8 {
                            let tile_size = board_rect.width()/8.0;
                            let x_pos = i as f32 * tile_size + board_rect.min.x;
                            let y_pos = j as f32 * tile_size + board_rect.min.y;
                            let color = if (i + j) % 2 == 0 {
                                Color32::from_rgb(210, 210, 155)
                            } else {
                                Color32::from_rgb(110, 85, 45)
                            };

                            let paint_rect =
                                egui::Rect {
                                    min: egui::pos2(x_pos, y_pos),
                                    max: egui::pos2(x_pos + tile_size,
                                                    y_pos + tile_size) };

                            shapes.push(
                                epaint::Shape::Rect(
                                    epaint::RectShape::filled(
                                        paint_rect,
                                        epaint::Rounding::none(),
                                        color
                                    )
                                )
                            );
                        }
                    }

                    ui.painter().extend(shapes);

                    if self.game_state.is_some() {
                        for i in 0..8 {
                            for j in 0..8 {
                                let tile_size = board_rect.width()/8.0;
                                let x_pos = j as f32 * tile_size + board_rect.min.x;
                                let y_pos = i as f32 * tile_size + board_rect.min.y;

                                let paint_rect =
                                    egui::Rect {
                                        min: egui::pos2(x_pos, y_pos),
                                        max: egui::pos2(x_pos + tile_size,
                                                        y_pos + tile_size) };

                                let img = match Player::try_from(self.game_state.as_ref().unwrap().board[i][j].get_owner()).unwrap() {
                                    Player::Black => {
                                        match PieceType::try_from(self.game_state.as_ref().unwrap().board[i][j].get_piece()).unwrap() {
                                            PieceType::Rook => &self.brook,
                                            PieceType::Knight => &self.bknight,
                                            PieceType::Bishop => &self.bbishop,
                                            PieceType::Queen => &self.bqueen,
                                            PieceType::King => &self.bking,
                                            PieceType::Pawn => &self.bpawn,
                                            PieceType::None => { continue; },
                                        }
                                    },
                                    Player::White => {
                                        match PieceType::try_from(self.game_state.as_ref().unwrap().board[i][j].get_piece()).unwrap() {
                                            PieceType::Rook => &self.wrook,
                                            PieceType::Knight => &self.wknight,
                                            PieceType::Bishop => &self.wbishop,
                                            PieceType::Queen => &self.wqueen,
                                            PieceType::King => &self.wking,
                                            PieceType::Pawn => &self.wpawn,
                                            PieceType::None => { continue; },
                                        }
                                    },
                                    _ => { continue; }
                                };

                                let final_img = egui::widgets::Image::new(img.texture_id(ctx),
                                                                          img.size_vec2());

                                final_img.paint_at(ui, paint_rect);
                            }
                        }
                    }
                });
            });
    }
}

