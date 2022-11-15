use std::fmt;
use bitfield::bitfield;
use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;
use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;

pub mod gui;

// TODO: make public API like GameState more apparent

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum PieceType {
    None,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Player {
    None,
    White,
    Black
}

bitfield!{
    #[derive(Copy, Clone)]
    struct Square(u8);
    impl Debug;
    get_piece, set_piece: 3, 0;
    get_owner, set_owner: 5, 4;
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut symbol = match PieceType::try_from(self.get_piece()).unwrap() {
            PieceType::Rook => 'R',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
            PieceType::Pawn => 'P',
            PieceType::None => '.',
        };
        if self.get_owner() == Player::Black as u8 {
            symbol = (symbol as u8 - 'A' as u8 + 'a' as u8) as char;
        }
        write!(f, "{}", symbol)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GameState {
    board: [[Square; 8]; 8],
    player_to_move: Player,
    castling_rights: CastlingRights,
    en_passant_target: [u8; 2],
    halfmove_counter: u16,
    fullmove_counter: u16,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..8 {
            for j in 0..8 {
                write!(f, "{} ", self.board[i][j])?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl GameState {
    fn set_piece_at(&mut self, row: usize, col: usize, piece: PieceType, owner: Player) -> () {
        assert_ne!(owner, Player::None);
        self.board[row][col].set_piece(piece.into());
        self.board[row][col].set_owner(owner.into());
    } 
}

bitfield!{
    #[derive(Copy, Clone)]
    struct CastlingRights(u8);
    impl Debug;
    get_black_queenside, set_black_queenside: 0;
    get_black_kingside, set_black_kingside: 1;
    get_white_queenside, set_white_queenside: 2;
    get_white_kingside, set_white_kingside: 3;
}

const FEN_INPUT: &str = "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 99 50";

// TODO: return an option or error result
pub fn fen_to_game_state(fen: String) -> GameState {
    println!("{}", FEN_INPUT);

    let mut game_state = GameState {
        board: [[Square(0); 8]; 8],
        player_to_move: Player::White,
        castling_rights: CastlingRights(0),
        en_passant_target: [0u8; 2],
        halfmove_counter: 0,
        fullmove_counter: 0,
    };

    let separator = Regex::new(r"([ ]+)").expect("Invalid regex");
    let splits: Vec<_> = separator.split(&fen).into_iter().collect();
    assert!(splits.len() >= 6);

    // section 0: pieces on the board
    let mut cur_row: usize = 0;
    let mut cur_col: usize = 0;
    for c in UnicodeSegmentation::graphemes(splits[0], true) {
        if c == "/" {
            cur_row += 1;
            cur_col = 0;
            continue;
        }
        match c {
            "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8" => cur_col += c.parse::<usize>().unwrap() - 1,
            "k" => game_state.set_piece_at(cur_row, cur_col, PieceType::King, Player::Black),
            "K" => game_state.set_piece_at(cur_row, cur_col, PieceType::King, Player::White),
            "b" => game_state.set_piece_at(cur_row, cur_col, PieceType::Bishop, Player::Black),
            "B" => game_state.set_piece_at(cur_row, cur_col, PieceType::Bishop, Player::White),
            "r" => game_state.set_piece_at(cur_row, cur_col, PieceType::Rook, Player::Black),
            "R" => game_state.set_piece_at(cur_row, cur_col, PieceType::Rook, Player::Black),
            "n" => game_state.set_piece_at(cur_row, cur_col, PieceType::Knight, Player::White),
            "N" => game_state.set_piece_at(cur_row, cur_col, PieceType::Knight, Player::White),
            "q" => game_state.set_piece_at(cur_row, cur_col, PieceType::Queen, Player::Black),
            "Q" => game_state.set_piece_at(cur_row, cur_col, PieceType::Queen, Player::White),
            "p" => game_state.set_piece_at(cur_row, cur_col, PieceType::Pawn, Player::Black),
            "P" => game_state.set_piece_at(cur_row, cur_col, PieceType::Pawn, Player::White),
            _ => panic!("Unexpected symbol in FEN input: {}", c),
        }
        cur_col += 1;
    }

    // section 1: player to move
    for c in UnicodeSegmentation::graphemes(splits[1], true) {
        match c {
            "w" => game_state.player_to_move = Player::White,
            "b" => game_state.player_to_move = Player::Black,
            _ => panic!("Unexpected symbol in FEN input: {}", c),
        }
    }

    // section 2: castling rights
    for c in UnicodeSegmentation::graphemes(splits[2], true) {
        match c {
            "q" => game_state.castling_rights.set_black_queenside(true),
            "Q" => game_state.castling_rights.set_white_queenside(true),
            "k" => game_state.castling_rights.set_black_kingside(true),
            "K" => game_state.castling_rights.set_white_kingside(true),
            "-" => {},
            _ => panic!("Unexpected symbol in FEN input: {}", c),
        }
    }

    // section 3: en passant target
    for c in UnicodeSegmentation::graphemes(splits[3], true) {
        match c {
            "a"|"b"|"c"|"d"|"e"|"f"|"g"|"h" => game_state.en_passant_target[1] = c.parse::<char>().unwrap() as u8 - 'a' as u8,
            "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8" => game_state.en_passant_target[0] = c.parse::<u8>().unwrap(),
            "-" => {},
            _ => panic!("Unexpected symbol in FEN input: {}", c),
        }
    }

    // section 4: halfmove counter
    game_state.halfmove_counter = splits[4].parse::<u16>().unwrap();

    // section 5: fullmove counter
    game_state.fullmove_counter = splits[5].parse::<u16>().unwrap();
 
    println!("{}", game_state);

    game_state
}

fn main() {
    gui::gui(fen_to_game_state(FEN_INPUT.to_string()), FEN_INPUT.to_string());
}
