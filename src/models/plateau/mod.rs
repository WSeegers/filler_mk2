mod parser;

use super::{constants, Piece, Player, Point};

use constants::*;

use std::fmt;

const DEFAULT_SIZE: usize = 50;
const DEFAULT_P1_START: Point = Point { x: 5, y: 5 };
const DEFAULT_P2_START: Point = Point { x: 44, y: 44 };

#[derive(Debug, Copy, Clone)]
enum Cell {
    Player1(bool),
    Player2(bool),
    Empty,
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        use Cell::*;
        match self {
            Player1(_) => match other {
                Player1(_) => true,
                _ => false,
            },
            Player2(_) => match other {
                Player2(_) => true,
                _ => false,
            },
            _ => self == other,
        }
    }
}

impl Cell {
    fn age(&self) -> Self {
        use Cell::*;
        match self {
            Player1(_) => Player1(false),
            Player2(_) => Player2(false),
            Empty => Empty,
        }
    }
}

#[derive(Debug)]
pub struct Plateau {
    player1_start: Point,
    player2_start: Point,
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    last_piece: Option<(Point, Piece)>,
}

impl Plateau {
    pub fn default() -> Self {
        Plateau::new(
            DEFAULT_SIZE,
            DEFAULT_SIZE,
            &DEFAULT_P1_START,
            &DEFAULT_P2_START,
        )
        .unwrap()
    }

    pub fn new(
        width: usize,
        height: usize,
        player1: &Point,
        player2: &Point,
    ) -> Result<Plateau, String> {
        let mut plateau = Plateau {
            player1_start: player1.clone(),
            player2_start: player2.clone(),
            width,
            height,
            cells: vec![Cell::Empty; (width * height) as usize],
            last_piece: None,
        };

        match plateau.is_in_bounds(player1) {
            true => plateau.set(player1, Cell::Player1(false)),
            false => return Err(String::from("Player1 out of bounds")),
        };

        match plateau.is_in_bounds(player2) {
            true => plateau.set(player2, Cell::Player2(false)),
            false => return Err(String::from("Player2 out of bounds")),
        };

        Ok(plateau)
    }

    pub fn is_in_bounds(&self, p: &Point) -> bool {
        p.x >= 0 && p.x < self.width as i32 && p.y >= 0 && p.y < self.height as i32
    }

    fn get(&self, p: &Point) -> Cell {
        match self.cells.get((self.width as i32 * p.y + p.x) as usize) {
            Some(c) => c.clone(),
            None => panic!("Cells incorrectly initialized"),
        }
    }

    fn set(&mut self, p: &Point, cell: Cell) {
        self.cells[(self.width as i32 * p.y + p.x) as usize] = cell;
    }

    fn is_valid_placement(
        &self,
        piece: &Piece,
        placement: &Point,
        owner: &Cell,
    ) -> Result<(), String> {
        let mut overlap = false;

        for y in 0..(piece.height()) as i32 {
            for x in 0..(piece.width()) as i32 {
                use Cell::{Empty, Player1, Player2};
                if !piece.get(Point { x, y }) {
                    continue;
                }

                let offset = &Point { x, y } + &placement;
                if !self.is_in_bounds(&offset) {
                    return Err(String::from("Piece out of bounds"));
                }

                let plat_cell = self.get(&offset);
                match plat_cell {
                    Empty => continue,
                    Player1(_) | Player2(_) if plat_cell == *owner => match overlap {
                        true => return Err(String::from("Overlap greater than one")),
                        false => overlap = true,
                    },
                    Player1(_) | Player2(_) => return Err(String::from("Overlap on other player")),
                }
            }
        }

        if !overlap {
            return Err(String::from("No Overlap"));
        }

        Ok(())
    }

    pub fn place_piece(
        &mut self,
        piece: &Piece,
        placement: &Point,
        player: Player,
    ) -> Result<(), String> {
        self.age_placement();
        let owner = match player {
            Player::Player1 => Cell::Player1(true),
            Player::Player2 => Cell::Player2(true),
        };

        self.is_valid_placement(piece, placement, &owner)?;

        for y in 0..(piece.height()) as i32 {
            for x in 0..(piece.width()) as i32 {
                if !piece.get(Point { x, y }) {
                    continue;
                }

                let offset = &Point { x, y } + &placement;
                self.set(&offset, owner);
            }
        }
        self.last_piece = Some((*placement, piece.clone()));

        Ok(())
    }

    fn age_placement(&mut self) {
        if let Some((placement, piece)) = self.last_piece.take() {
            for y in 0..(piece.height()) as i32 {
                for x in 0..(piece.width()) as i32 {
                    if !piece.get(Point { x, y }) {
                        continue;
                    }
                    let offset = &Point { x, y } + &placement;
                    let owner = self.get(&offset);
                    self.set(&offset, owner.age());
                }
            }
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn player_start(&self, player: Player) -> Point {
        match player {
            Player::Player1 => self.player1_start,
            Player::Player2 => self.player2_start,
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cell = match self {
            Cell::Player1(is_new) => match *is_new {
                true => PLAYER1_NEW,
                false => PLAYER1,
            },
            Cell::Player2(is_new) => match *is_new {
                true => PLAYER2_NEW,
                false => PLAYER2,
            },
            Cell::Empty => EMPTY,
        };
        write!(f, "{}", cell)
    }
}

impl fmt::Display for Plateau {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Plateau {} {}:", self.height, self.width)?;
        write!(f, "    ")?;
        for y in 0..(self.width) {
            write!(f, "{}", y % 10)?;
        }
        writeln!(f, "")?;

        for y in 0..(self.height) as i32 {
            write!(f, "{:03} ", y)?;
            for x in 0..(self.width) as i32 {
                let cell = self.get(&Point { x, y });
                write!(f, "{}", cell)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece_horizontal() -> Piece {
        let cells = vec![false, true, true, false];
        let width = 4;
        let height = 1;
        Piece::new(width, height, cells)
    }

    fn piece_vertical() -> Piece {
        let cells = vec![false, true, true, false];
        let width = 1;
        let height = 4;
        Piece::new(width, height, cells)
    }

    fn piece_square() -> Piece {
        let mut cells = vec![true; 9];
        cells[4] = false;
        let width = 3;
        let height = 3;
        Piece::new(width, height, cells)
    }

    #[test]
    fn good_placement_horizontal_with_overlap() {
        let player_1_start = Point::new(1, 1);
        let plateau = Plateau::new(3, 3, &player_1_start, &Point::new(2, 2)).unwrap();
        let piece = piece_horizontal();

        print!("{}", plateau);
        println!("{}", piece);

        let placement = Point::new(0, 1);
        println!("placement: {:?}", placement);
        assert_eq!(
            plateau.is_valid_placement(&piece, &placement, &Cell::Player1(false)),
            Ok(())
        );

        let placement = Point::new(-1, 1);
        println!("Placement: {:?}", placement);
        assert_eq!(
            plateau.is_valid_placement(&piece, &placement, &Cell::Player1(false)),
            Ok(())
        );
    }

    #[test]
    fn good_placement_vertical_with_overlap() {
        let player_1_start = Point::new(1, 1);
        let plateau = Plateau::new(3, 3, &player_1_start, &Point::new(2, 2)).unwrap();
        let piece = piece_vertical();

        print!("{}", plateau);
        println!("{}", piece);

        let placement = Point::new(1, 0);
        println!("placement: {:?}", placement);
        assert_eq!(
            plateau.is_valid_placement(&piece, &placement, &Cell::Player1(false)),
            Ok(())
        );

        let placement = Point::new(1, -1);
        println!("Placement: {:?}", placement);
        assert_eq!(
            plateau.is_valid_placement(&piece, &placement, &Cell::Player1(false)),
            Ok(())
        );
    }

    #[test]
    fn good_placement_wrap() {
        let player_1_start = Point::new(1, 1);
        let plateau = Plateau::new(3, 3, &player_1_start, &Point::new(2, 2)).unwrap();
        let piece = piece_square();

        print!("{}", plateau);
        println!("{}", piece);

        let placement = Point::new(0, 0);
        println!("placement: {:?}", placement);
        assert_eq!(
            plateau.is_valid_placement(&piece, &placement, &Cell::Player2(false)),
            Ok(())
        );
    }

    #[test]
    fn default_should_not_panic() {
        Plateau::default();
    }
}
