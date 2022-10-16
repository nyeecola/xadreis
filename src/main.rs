use std::fmt;
use bitfield::bitfield;
use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum PieceType {
    NONE,
    PAWN,
    ROOK,
    KNIGHT,
    BISHOP,
    QUEEN,
    KING,
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Player {
    NONE,
    WHITE,
    BLACK
}

bitfield!{
    #[derive(Copy, Clone)]
    struct Square(u8);
    impl Debug;
    get_piece, set_piece: 3, 0;
    get_owner, set_owner: 5, 3;
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut symbol = match PieceType::try_from(self.get_piece()).unwrap() {
            PieceType::ROOK => 'R',
            PieceType::KNIGHT => 'N',
            PieceType::BISHOP => 'B',
            PieceType::QUEEN => 'Q',
            PieceType::KING => 'K',
            PieceType::PAWN => 'P',
            PieceType::NONE => '.',
        };
        if self.get_owner() == Player::BLACK as u8 {
            symbol = (symbol as u8 - 'A' as u8 + 'a' as u8) as char;
        }
        write!(f, "{}", symbol)?;
        Ok(())
    }
}

#[derive(Debug)]
struct GameState {
    board: [[Square; 8]; 8],
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
        assert_ne!(owner, Player::NONE);
        self.board[row][col].set_piece(piece as u8);
        self.board[row][col].set_owner(owner as u8);
    } 
}

const FEN_INPUT: &str = "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 99 50";

fn main() {
    println!("{}", FEN_INPUT);

    let mut game_state = GameState {
        board: [[Square(0); 8]; 8],
    };

    game_state.set_piece_at(0, 3, PieceType::KNIGHT, Player::BLACK);

    println!("{}", game_state);
}
