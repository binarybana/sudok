#![feature(vec_remove_item)]

use std::io::{BufRead, BufReader};


#[derive(Debug,Clone,PartialEq)]
struct GuessCell {
    possibles: Vec<u8>,
}

impl GuessCell {
    fn new() -> GuessCell {
        let mut set = Vec::with_capacity(10);
        for i in 1..10 {
            set.push(i);
        }
        GuessCell{possibles:set}
    }
}

#[derive(Debug,Clone,PartialEq)]
enum Cell {
    Unknown(GuessCell),
    Known(u8),
}

#[derive(Debug,Clone)]
struct Puzzle(Vec<Cell>);

impl Puzzle {
    fn row(&mut self, row: u8, i: u8) -> &mut Cell {
        assert!(row <9 && i <9);
        &mut self.0[(row*9+i) as usize]
    }

    fn col(&mut self, col: u8, i: u8) -> &mut Cell {
        assert!(col <9 && i <9);
        &mut self.0[(i*9+col) as usize]
    }

    fn subcell(&mut self, index: u8, subindex: u8) -> &mut Cell {
        assert!(index <9 && subindex <9);
        let rowblock = index/3;
        let colblock = index%3;
        let rowsub = subindex/3;
        let colsub = subindex%3;
        let row = rowblock*3+rowsub;
        self.row(row, colblock*3+colsub)
    }

    fn is_done(&self) -> bool {
        for cell in self.0.iter() {
            if let &Cell::Unknown(_) = cell {
                return false;
            }
        }
        true
    }

    fn get_most_constrained(&mut self) -> &mut Cell {
        assert!(!self.is_done());
        // TODO: rewrite using functional operators
        let mut min_options = 11;
        let mut min_ind = 0;
        for (i, cell) in self.0.iter().enumerate() {
            if let &Cell::Unknown(val) = cell {
                if val.possibles.len() < min_options {
                    min_options = val.possibles.len();
                    min_ind = i;
                }
            }
        }
        &mut self.0[min_ind]
    }

    fn with_cell_choice(&self, pivot_cell_index: usize, val: u8) -> Puzzle {
        let mut new_puzzle = self.clone();
        new_puzzle.0[pivot_cell_index] = Cell::Known(val);
        new_puzzle
    }
}

fn parse_puzzle(flat: &str) -> Puzzle {
    let mut cells = Vec::new();
    for c in flat.chars() {
        cells.push(if c == '.' 
                       {Cell::Unknown(GuessCell::new())}
                   else
                       {Cell::Known(c.to_digit(10).unwrap() as u8)})
    }
    Puzzle(cells)
}


fn update_constraints(puzzle: &mut Puzzle) {
    // puzzle.0[1] = Cell::Known(3);
    let mut set = Vec::with_capacity(10);
    for row in 0..9 {
        // collect known values in this row
        for elem in 0..9 {
            if let Cell::Known(val) = *puzzle.row(row, elem) {
                set.push(val);
            }
        }
        // Now remove that from possible set list
        for elem in 0..9 {
            if let &mut Cell::Unknown(ref mut val) = puzzle.row(row, elem) {
                for item in set.iter() {
                    val.possibles.remove_item(&item);
                }
            }
        }
        set.clear();
    }
    for col in 0..9 {
        // collect known values in this col
        for elem in 0..9 {
            if let Cell::Known(val) = *puzzle.col(col, elem) {
                set.push(val);
            }
        }
        // Now remove that from possible set list
        for elem in 0..9 {
            if let &mut Cell::Unknown(ref mut val) = puzzle.col(col, elem) {
                for item in set.iter() {
                    val.possibles.remove_item(&item);
                }
            }
        }
        set.clear();
    }
    for block in 0..9 {
        // collect known values in this block
        for elem in 0..9 {
            if let Cell::Known(val) = *puzzle.subcell(block, elem) {
                set.push(val);
            }
        }
        // Now remove that from possible set list
        for elem in 0..9 {
            if let &mut Cell::Unknown(ref mut val) = puzzle.subcell(block, elem) {
                for item in set.iter() {
                    val.possibles.remove_item(&item);
                }
                println!("{:?}", val.possibles);
            }
        }
        set.clear();
    }
}

fn solve_puzzle(puzzle: Puzzle) -> Puzzle {
    let mut result = puzzle.clone();
    update_constraints(&mut result);
    fn inner_solve(puzzle: Puzzle) -> Option<Puzzle> {
        // Solve this puzzle as much as possible
        while let Some(choice) = puzzle.get_forced_choice() {
            make_choice(&mut puzzle, choice);
            update_constraints(&mut puzzle);
        }
        // Now either pop back up the stack, or randomly guess all
        // possible choices of the most constrained single cell
        if puzzle.is_done() {
            return Some(puzzle);
        }
        let pivot_cell_index = puzzle.get_most_constrained();
        let possibles = puzzle.0[pivot_cell_index] FIXME
        // FIXME: need to grab possibles out and index (for with_cell_choice)

        for val in pivot_cell.possibles.iter().enumerate() {
            let new_puzzle = puzzle.with_cell_choice(pivot_cell_index, val);
            if let Some(soln) = inner_solve(new_puzzle) {
                return Some(soln);
            }
        }
        return None;
    }
    // FIXME
    inner_solve(result).unwrap()
}

fn main() {

    let fname = match std::env::args().skip(1).next() {
        Some(file) => file,
        None => panic!("Need a file as argument"),
    };
    let fid = std::fs::File::open(fname).expect("Not a valid filename");
    for line in BufReader::new(fid).lines() {
        let puzzle = parse_puzzle(&line.unwrap());
        let soln = solve_puzzle(puzzle.clone());
        println!("{:?}\n{:?}", puzzle, soln);
        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_puzzle() -> Puzzle {
        // let c = "123456789123456789123456789123456789123456789123456789123456789123456789123456789
        // let r = "111111111222222222333333333444444444555555555666666666777777777888888888999999999
        // let s = "111222333111222333111222333444555666444555666444555666777888999777888999777888999
        // let s = "9.4..5...25.6..1..31......8.7...9...4..26......147....7.......2...3..8.6.4.....9.";
        let s = "9.4..5...25.6..1..31......8.7...9...4..26......147....7.......2...3..8.6.4.....9.";
        parse_puzzle(s)
    }

    #[test]
    fn parsing() {
        let p = get_puzzle();
        assert_eq!(p.0[0], Cell::Known(9));
        use std::mem;
        assert_eq!(mem::discriminant(&p.0[1]), mem::discriminant(&Cell::Unknown(GuessCell::new())));
        assert_eq!(p.0[2], Cell::Known(4));
    }

    #[test]
    fn constraint_update() {
        let mut p = get_puzzle();
        update_constraints(&mut p);
    }

    #[test]
    fn indexing() {
        let mut p = get_puzzle();
        assert_eq!(*p.row(0, 2), Cell::Known(4));
        assert_eq!(*p.row(1, 1), Cell::Known(5));

        assert_eq!(*p.col(0, 1), Cell::Known(2));
        assert_eq!(*p.col(2, 5), Cell::Known(1));

        assert_eq!(*p.subcell(2, 3), Cell::Known(1));
    }
}

