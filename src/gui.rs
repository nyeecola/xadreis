#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::generate_legal_moves;
use crate::perft;
use egui_extras::Size;
use egui_extras::TableBuilder;
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

pub fn gui(game_state: Box<GameState>, fen: String) {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(emath::Vec2{x:1200.0,y:800.0});
    let app = XadreisGUI::from_game_state(game_state, fen);
    eframe::run_native(
        "Xadreis",
        options,
        Box::new(|_cc| Box::new(app)),
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

    game_state: Option<Box<GameState>>,
    fen: String,

    perft: Option<[isize; 8]>,
}

impl XadreisGUI {
    fn from_game_state(game_state: Box<GameState>, fen: String) -> Self {
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

            perft: None,
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
                    self.game_state = Some(Box::new(fen_to_game_state(self.fen.to_string())));
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
                                Color32::from_rgb(205, 180, 145)
                            } else {
                                Color32::from_rgb(135, 105, 65)
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

                                let mut offset = egui::vec2(tile_size * 0.2, tile_size * 0.2);

                                let img = match Player::try_from(self.game_state.as_ref().unwrap().board[i][j].get_owner()).unwrap() {
                                    Player::Black => {
                                        match PieceType::try_from(self.game_state.as_ref().unwrap().board[i][j].get_piece()).unwrap() {
                                            PieceType::Rook => &self.brook,
                                            PieceType::Knight => &self.bknight,
                                            PieceType::Bishop => &self.bbishop,
                                            PieceType::Queen => &self.bqueen,
                                            PieceType::King => &self.bking,
                                            PieceType::Pawn => { offset.x *= 1.2; &self.bpawn },
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
                                            PieceType::Pawn => { offset.x *= 1.2; &self.wpawn },
                                            PieceType::None => { continue; },
                                        }
                                    },
                                    _ => { continue; }
                                };

                                let paint_rect =
                                    egui::Rect {
                                        min: egui::pos2(x_pos, y_pos) + offset,
                                        max: egui::pos2(x_pos + tile_size,
                                                        y_pos + tile_size) - offset
                                    };

                                let final_img = egui::widgets::Image::new(img.texture_id(ctx),
                                                                          img.size_vec2());

                                final_img.paint_at(ui, paint_rect);
                            }
                        }
                    }
                });
            });
        Window::new("Perft")
            .open(&mut open)
            .show(ctx, |top_ui| {
                if top_ui.button("Run perft() for current board position").clicked() {
                    // TODO:
                    //  - understand this & + as_ref() stuff
                    //  - stop unwrap()'ing here, we could very easily crash
                    //println!("Perft(1) moves: {:?}", generate_legal_moves(&self.game_state.as_ref().unwrap()));
                    let mut perft_results = [-1isize; 8];
                    Some(perft(&mut perft_results, &self.game_state.as_ref().unwrap(), 3));
                    self.perft = Some(perft_results);
                }

                let table = TableBuilder::new(top_ui)
                    .striped(true)
                    .cell_layout(egui::Layout::right_to_left(egui::Align::Center))
                    .columns(Size::initial(60.0).at_least(40.0), 7)
                    .column(Size::remainder().at_least(60.0))
                    .resizable(true);

                table
                    .header(30.0, |mut header| {
                        header.col(|hui| {hui.heading("");});
                        for i in 1..8 {
                            header.col(|hui| {hui.heading(format!("Perft({})", i));});
                        }
                    })
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|rui| { rui.label("Nodes"); });
                            for i in 1..8 {
                                if self.perft.is_some() {
                                    let count = self.perft.unwrap()[i];

                                    if count != -1 {
                                        row.col(|rui| { rui.label(count.to_string()); });
                                    } else {
                                        row.col(|rui| { rui.label("??"); });
                                    }
                                } else {
                                    row.col(|rui| { rui.label("??"); });
                                }
                            }
                        });
                    });
                });
    }
}

