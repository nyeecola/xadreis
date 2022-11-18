use std::fmt;
use bitfield::bitfield;
use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;
use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;

pub mod gui;

// TODO: make public API like GameState and fen_to_game_state more apparent

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Copy, Clone)]
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

impl PieceType {
    fn iterator() -> impl Iterator<Item = PieceType> {
        [PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen].iter().copied()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
enum MoveSideEffectType {
    MovePiece,
    RemovePiece,
    AddPiece,
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub struct GameState {
    board: [[Square; 8]; 8],
    player_to_move: Player,
    castling_rights: CastlingRights,
    en_passant_target: Option<(u8, u8)>,
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
        assert!((owner == Player::None && piece == PieceType::None) ||
                (owner != Player::None && piece != PieceType::None));
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

#[derive(Debug, Clone, Copy)]
pub struct MoveSideEffect {
    effect_type: MoveSideEffectType,
    new_piece_type: Option<PieceType>, // only used for `AddPiece`
    // (line, column) where (0,0) is black's rook and white king is at (7,4)
    from: Option<(usize, usize)>,
    to: Option<(usize, usize)>,
}

// TODO: pack this more to save memory later on
#[derive(Debug, Clone, Copy)]
pub struct Move {
    // (line, column) where (0,0) is black's rook and white king is at (7,4)
    from: (usize, usize),
    to: (usize, usize),
    side_effect: Option<MoveSideEffect>
}

const FEN_INPUT: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// TODO: return an option or error result
pub fn fen_to_game_state(raw_fen: String) -> GameState {
    println!("{}", FEN_INPUT);

    let fen = raw_fen.trim();

    let mut game_state = GameState {
        board: [[Square(0); 8]; 8],
        player_to_move: Player::White,
        castling_rights: CastlingRights(0),
        en_passant_target: None,
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
            "R" => game_state.set_piece_at(cur_row, cur_col, PieceType::Rook, Player::White),
            "n" => game_state.set_piece_at(cur_row, cur_col, PieceType::Knight, Player::Black),
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
        let (mut file, mut row) = (0, 0);
        match c {
            "a"|"b"|"c"|"d"|"e"|"f"|"g"|"h" => file = c.parse::<char>().unwrap() as u8 - 'a' as u8,
            "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8" => row = c.parse::<u8>().unwrap(),
            "-" => { break; },
            _ => panic!("Unexpected symbol in FEN input: {}", c),
        }
        game_state.en_passant_target = Some((row, file));
    }

    // section 4: halfmove counter
    game_state.halfmove_counter = splits[4].parse::<u16>().unwrap();

    // section 5: fullmove counter
    game_state.fullmove_counter = splits[5].parse::<u16>().unwrap();
 
    println!("{}", game_state);

    game_state
}

fn generate_rook_attacks(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    let mut moves = vec![];

    let board = game_state.board;

    for o in y+1..8 {
        let target = board[o][x];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (o,x), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }
    for o in (0..y).rev() {
        let target = board[o][x];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (o,x), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }
    for o in x+1..8 {
        let target = board[y][o];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (y,o), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }
    for o in (0..x).rev() {
        let target = board[y][o];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (y,o), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }

    moves
}

fn generate_rook_moves(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    generate_rook_attacks(game_state, owner, x, y)
}

fn generate_bishop_attacks(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    let mut moves = vec![];

    let board = game_state.board;

    for o in 1..8 {
        if y+o > 7 || x+o > 7 {
            break;
        }
        let target = board[y+o][x+o];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (y+o,x+o), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }
    for o in 1..8 {
        if y as isize - o < 0 || x as isize - o < 0 {
            break;
        }
        let target = board[y-o as usize][x-o as usize];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (y-o as usize, x-o as usize), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }
    for o in 1..8 as isize {
        if y + o as usize > 7 || x as isize - o < 0 {
            break;
        }
        let target = board[y+o as usize][x-o as usize];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (y+o as usize,x-o as usize), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }
    for o in 1..8 as isize {
        if y as isize - o < 0 || x as isize + o > 7 {
            break;
        }
        let target = board[y-o as usize][x+o as usize];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner != owner {
            moves.push(Move {from: (y,x), to: (y-o as usize,x+o as usize), side_effect: None})
        }
        if target_owner != Player::None {
            break;
        }
    }

    moves
}

fn generate_bishop_moves(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    generate_bishop_attacks(game_state, owner, x, y)
}

fn generate_pawn_attacks(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    let mut moves = vec![];

    let board = game_state.board;

    let mut sign = 1isize;
    if owner == Player::White {
        sign = -1;
    }

    {
        let o = y as isize + sign;
        if (o >= 0 && o <= 7) && x >= 1 { 
            let target = board[o as usize][x-1];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != Player::None && target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize,x-1), side_effect: None});
            }
        }
    }

    {
        let o = y as isize + sign;
        if (o >= 0 && o <= 7) && x+1 <= 7 { 
            let target = board[o as usize][x+1];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != Player::None && target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize,x+1), side_effect: None});
            }
        }
    }

    moves
}

fn generate_pawn_moves(game_state: &Box<GameState>, player: Player, x: usize, y: usize) -> Vec<Move> {
    let mut moves = vec![];

    moves.extend(generate_pawn_attacks(game_state, player, x, y));

    let board = game_state.board;

    let mut sign = 1isize;
    if player == Player::White {
        sign = -1;
    }

    for i in 1..3isize {
        let o = y as isize + sign * i;
        if o < 0 || o > 7  {
            break;
        }
        if i == 2 {
            if (player == Player::White && y != 6) ||
               (player == Player::Black && y != 1) {
                break;
            }
        }
        let target = board[o as usize][x];
        let target_owner = Player::try_from(target.get_owner()).unwrap();
        if target_owner == Player::None {
            let mut mv = Move {from: (y,x), to: (o as usize, x), side_effect: None};
            if i == 1 {
                if (player == Player::White && y == 0) ||
                   (player == Player::Black && y == 7) {
                    for pt in PieceType::iterator() {
                        mv.side_effect = Some(MoveSideEffect{effect_type: MoveSideEffectType::AddPiece,
                                                             new_piece_type: Some(pt), from: None,
                                                             to: Some((o as usize, x))});
                    }
                }
            };
            moves.push(mv);
        } else {
            break;
        }
    }

    if game_state.en_passant_target.is_some() {
        let target = game_state.en_passant_target.unwrap();

        let t_y = target.0 as isize;
        let t_x = target.1 as isize;

        if t_y == y as isize && (t_x-1 == x as isize || t_x+1 == x as isize) {
            let o = t_y + sign;
            if o >= 0 && o <= 7  {
                moves.push(Move {from: (y,x), to: (o as usize, t_x as usize),
                                 side_effect: Some(MoveSideEffect{effect_type: MoveSideEffectType::RemovePiece,
                                                                  new_piece_type: None,
                                                                  from: Some((t_y as usize, t_x as usize)), to: None})});
            }
        }
    }

    moves
}

fn generate_knight_attacks(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    let mut moves = vec![];

    let board = game_state.board;

    {
        let o = y as isize + 2;
        let p = x as isize - 1;
        if o <= 7 && p >= 0 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }
    {
        let o = y as isize + 2;
        let p = x as isize + 1;
        if o <= 7 && p <= 7 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }
    {
        let o = y as isize - 2;
        let p = x as isize - 1;
        if o >= 0 && p >= 0 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }
    {
        let o = y as isize - 2;
        let p = x as isize + 1;
        if o >= 0 && p <= 7 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }

    {
        let o = y as isize + 1;
        let p = x as isize - 2;
        if o <= 7 && p >= 0 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }
    {
        let o = y as isize + 1;
        let p = x as isize + 2;
        if o <= 7 && p <= 7 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }
    {
        let o = y as isize - 1;
        let p = x as isize - 2;
        if o >= 0 && p >= 0 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }
    {
        let o = y as isize - 1;
        let p = x as isize + 2;
        if o >= 0 && p <= 7 { 
            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y,x), to: (o as usize, p as usize), side_effect: None});
            }
        }
    }

    moves
}

fn generate_knight_moves(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    generate_knight_attacks(game_state, owner, x, y)
}

fn generate_king_attacks(game_state: &Box<GameState>, owner: Player, x: isize, y: isize) -> Vec<Move> {
    let mut moves = vec![];

    let board = game_state.board;

    for o in y-1..y+2 {
        for p in x-1..x+2 {
            if o == y && p == x {
                continue;
            }

            if o < 0 || o > 7  {
                continue;
            }

            if p < 0 || p > 7  {
                continue;
            }

            let target = board[o as usize][p as usize];
            let target_owner = Player::try_from(target.get_owner()).unwrap();
            if target_owner != owner {
                moves.push(Move {from: (y as usize,x as usize),
                                 to: (o as usize, p as usize),
                                 side_effect: None});
            }
        }
    }

    moves
}

fn generate_king_moves(game_state: &Box<GameState>, owner: Player, x: isize, y: isize, is_in_check: bool) -> Vec<Move> {
    let mut moves = vec![];

    moves.extend(generate_king_attacks(game_state, owner, x, y));

    let board = game_state.board;

    if !is_in_check {
        match owner {
            Player::Black => {
                if game_state.castling_rights.get_black_kingside() {
                    if PieceType::try_from(board[0][5].get_piece()).unwrap() == PieceType::None &&
                        PieceType::try_from(board[0][6].get_piece()).unwrap() == PieceType::None {
                        moves.push(Move {from: (0,4), to: (0,6),
                                         side_effect: Some(MoveSideEffect{effect_type: MoveSideEffectType::MovePiece, new_piece_type: None, from: Some((0,7)), to: Some((0,5))})});
                    }
                }
                if game_state.castling_rights.get_black_queenside() {
                    if PieceType::try_from(board[0][3].get_piece()).unwrap() == PieceType::None &&
                        PieceType::try_from(board[0][2].get_piece()).unwrap() == PieceType::None &&
                        PieceType::try_from(board[0][1].get_piece()).unwrap() == PieceType::None {
                        moves.push(Move {from: (0,4), to: (0,2),
                                         side_effect: Some(MoveSideEffect{effect_type: MoveSideEffectType::MovePiece, new_piece_type: None, from: Some((0,0)), to: Some((0,3))})});
                    }
                }
            },
            Player::White => {
                if game_state.castling_rights.get_black_kingside() {
                    if PieceType::try_from(board[7][5].get_piece()).unwrap() == PieceType::None &&
                        PieceType::try_from(board[7][6].get_piece()).unwrap() == PieceType::None {
                        moves.push(Move {from: (7,4), to: (7,6),
                                         side_effect: Some(MoveSideEffect{effect_type: MoveSideEffectType::MovePiece, new_piece_type: None, from: Some((7,7)), to: Some((7,5))})});
                    }
                }
                if game_state.castling_rights.get_black_queenside() {
                    if PieceType::try_from(board[7][3].get_piece()).unwrap() == PieceType::None &&
                        PieceType::try_from(board[7][2].get_piece()).unwrap() == PieceType::None &&
                        PieceType::try_from(board[7][1].get_piece()).unwrap() == PieceType::None {
                        moves.push(Move {from: (7,4), to: (7,2),
                                         side_effect: Some(MoveSideEffect{effect_type: MoveSideEffectType::MovePiece, new_piece_type: None, from: Some((7,7)), to: Some((7,3))})});
                    }
                }
            },
            _ => { panic!{"Invalid player at generate_king_moves!"}; }
        }
    }

    moves
}

fn generate_queen_attacks(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    let mut moves = vec![];

    moves.extend(generate_rook_moves(game_state, owner, x, y));
    moves.extend(generate_bishop_moves(game_state, owner, x, y));

    moves
}

fn generate_queen_moves(game_state: &Box<GameState>, owner: Player, x: usize, y: usize) -> Vec<Move> {
    generate_queen_attacks(game_state, owner, x, y)
}

fn swap_player_turn(game_state: &mut Box<GameState>) {
    match game_state.player_to_move {
        Player::Black => { game_state.player_to_move = Player::White; },
        Player::White => { game_state.player_to_move = Player::Black; },
        Player::None => { panic!("Invalid player_to_move"); }
    };
}

fn make_move(game_state: &mut Box<GameState>, mv: Move) {
    {
        let piece = PieceType::try_from(game_state.board[mv.from.0][mv.from.1].get_piece()).unwrap();
        let owner = Player::try_from(game_state.board[mv.from.0][mv.from.1].get_owner()).unwrap();

        game_state.set_piece_at(mv.to.0, mv.to.1, piece, owner);
        game_state.set_piece_at(mv.from.0, mv.from.1, PieceType::None, Player::None);
    }

    if mv.side_effect.is_some() {
        let smv = mv.side_effect.unwrap();

        match smv.effect_type {
            MoveSideEffectType::MovePiece => {
                let from = smv.from.unwrap();
                let to = smv.to.unwrap();

                let piece = PieceType::try_from(game_state.board[from.0][from.1].get_piece()).unwrap();
                let owner = Player::try_from(game_state.board[from.0][from.1].get_owner()).unwrap();

                game_state.set_piece_at(to.0, to.1, piece, owner);
                game_state.set_piece_at(from.0, from.1, PieceType::None, Player::None);
            },
            MoveSideEffectType::AddPiece => {
                let to = smv.to.unwrap();
                let new_piece = smv.new_piece_type.unwrap();
                let owner = Player::try_from(game_state.board[mv.from.0][mv.from.1].get_owner()).unwrap();

                game_state.set_piece_at(to.0, to.1, new_piece, owner);
            }
            MoveSideEffectType::RemovePiece => {
                let from = smv.from.unwrap();

                game_state.set_piece_at(from.0, from.1, PieceType::None, Player::None);
            }
        };

    }

    swap_player_turn(game_state);
}

fn is_in_check(game_state: &Box<GameState>, player: Player, moves: Vec<Move>) -> bool {
    let board = game_state.board;

    for mv in moves {
        let square = board[mv.to.0][mv.to.1];
        let target_owner = Player::try_from(square.get_owner()).unwrap();
        let target_piece = PieceType::try_from(square.get_piece()).unwrap();

        if target_owner == player && target_piece == PieceType::King {
            return true;
        }
    }

    return false;
}

fn generate_attacks(game_state: &Box<GameState>, player: Player) -> Vec<Move> {
    let mut attacks = vec![];

    let board = game_state.board;

    for y in 0..8 {
        for x in 0..8 {
            let square = board[y][x];
            let owner = Player::try_from(square.get_owner()).unwrap();
            let piece = PieceType::try_from(square.get_piece()).unwrap();

            if player == owner {
                match piece {
                    PieceType::Rook => {
                        attacks.extend(generate_rook_attacks(&game_state, owner, x, y));
                    },
                    PieceType::Knight => {
                        attacks.extend(generate_knight_attacks(&game_state, owner, x, y));
                    },
                    PieceType::Bishop => {
                        attacks.extend(generate_bishop_attacks(&game_state, owner, x, y));
                    },
                    PieceType::Queen => {
                        attacks.extend(generate_queen_attacks(&game_state, owner, x, y));
                    },
                    PieceType::King => {
                        attacks.extend(generate_king_attacks(&game_state, owner, x as isize, y as isize));
                    },
                    PieceType::Pawn => {
                        attacks.extend(generate_pawn_attacks(&game_state, owner, x, y));
                    },
                    PieceType::None => { continue; },
                }
            }
        }
    }

    attacks
} 

// TODO: measure this and make it faster
// TODO: optimize this
fn generate_legal_moves(game_state: &Box<GameState>) -> Vec<Move> {
    let mut moves = vec![];

    let board = game_state.board;

    let opp_attacks = if game_state.player_to_move == Player::Black {
        generate_attacks(game_state, Player::White)
    } else {
        assert!(game_state.player_to_move == Player::White);
        generate_attacks(game_state, Player::Black)
    };

    let player_in_check = is_in_check(game_state, game_state.player_to_move, opp_attacks);

    for y in 0..8 {
        for x in 0..8 {
            let square = board[y][x];
            let owner = Player::try_from(square.get_owner()).unwrap();
            let piece = PieceType::try_from(square.get_piece()).unwrap();

            if game_state.player_to_move == owner {
                match piece {
                    PieceType::Rook => {
                        moves.extend(generate_rook_moves(&game_state, owner, x, y));
                    },
                    PieceType::Knight => {
                        moves.extend(generate_knight_moves(&game_state, owner, x, y));
                    },
                    PieceType::Bishop => {
                        moves.extend(generate_bishop_moves(&game_state, owner, x, y));
                    },
                    PieceType::Queen => {
                        moves.extend(generate_queen_moves(&game_state, owner, x, y));
                    },
                    PieceType::King => {
                        moves.extend(generate_king_moves(&game_state, owner, x as isize, y as isize, player_in_check));
                    },
                    PieceType::Pawn => {
                        moves.extend(generate_pawn_moves(&game_state, owner, x, y));
                    },
                    PieceType::None => { continue; },
                }
            }
        }
    }

    let mut final_moves = vec![];

    // remove moves that would leave the player in check
    // TODO: optimize move undoing
    for mv in &moves {
        let mut tmp_game_state = game_state.clone();
        make_move(&mut tmp_game_state, *mv);
        //println!("tmp_game_state\n{}", tmp_game_state);
        let tmp_attacks = generate_attacks(&tmp_game_state, tmp_game_state.player_to_move);
        //println!("tmp_attacks\n{:?}", tmp_attacks);
        if !is_in_check(&tmp_game_state, game_state.player_to_move, tmp_attacks) {
            final_moves.push(*mv);
        }
    }

    final_moves
}

pub fn perft(results: &mut [isize; 8], game_state: &Box<GameState>, n: usize) -> usize {
    perft_imp(results, game_state, n, 1)
}

fn perft_imp(results: &mut [isize; 8], game_state: &Box<GameState>, n: usize, level: usize) -> usize {
    if n == 0 {
        return 1;
    }

    let mut count = 0;
    let moves = generate_legal_moves(game_state);

    // TODO: optimize this
    for mv in &moves {
        let mut tmp_game_state = game_state.clone();
        make_move(&mut tmp_game_state, *mv);
        //println!("{}", tmp_game_state);
        count += perft_imp(results, &tmp_game_state, n - 1, level + 1);
    }

    if results[level] == -1 {
        results[level] = 0;
    }
    results[level] += moves.len() as isize;

    count
}

fn main() {
    let game_state = Box::new(fen_to_game_state(FEN_INPUT.to_string()));

    gui::gui(game_state.clone(), FEN_INPUT.to_string());
}
