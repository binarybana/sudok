#![feature(vec_remove_item)]

#[macro_use] extern crate log;
extern crate env_logger;

use std::io::{BufRead, BufReader};

extern crate fixedbitset;
use fixedbitset::FixedBitSet;

#[derive(Debug,Clone,PartialEq)]
struct GuessCell {
    possibles: FixedBitSet,
}

impl GuessCell {
    fn new() -> GuessCell {
        // We eat the 1 bit waste for simpler 0 to 1 based indexing in Sudoku
        let mut set = FixedBitSet::with_capacity(10);
        set.insert_range(1..10);
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

    fn get_forced_choice(&mut self) -> Option<(usize, u8)> {
        for (i, cell) in self.0.iter().enumerate() {
            if let &Cell::Unknown(ref val) = cell {
                if val.possibles.count_ones(..) == 1 {
                    return Some((i, val.possibles.ones().next().unwrap() as u8));
                }
            }
        }
        None
    }

    fn get_most_constrained(&mut self) -> (usize, &mut Cell) {
        assert!(!self.is_done());
        // TODO: rewrite using functional operators
        debug!("most constrained: \n{}", self);
        debug!("{:?}", self.0);
        let mut min_options = 11;
        let mut min_ind = 0;
        for (i, cell) in self.0.iter().enumerate() {
            if let &Cell::Unknown(ref val) = cell {
                debug!("Considering index {} which has {} options", i, val.possibles.count_ones(..));
                if val.possibles.count_ones(..) < min_options {
                    min_options = val.possibles.count_ones(..);
                    min_ind = i;
                }
            }
        }
        debug!("Just found most constrained cell at index {} with {} options", min_ind, min_options);
        (min_ind, &mut self.0[min_ind])
    }

    fn with_cell_choice(&self, pivot_cell_index: usize, val: u8) -> Puzzle {
        let mut new_puzzle = self.clone();
        new_puzzle.0[pivot_cell_index] = Cell::Known(val);
        new_puzzle
    }

    fn is_valid(&self) -> bool {
        let mut myself = self.clone();
        if !self.is_done() {
            debug!("Not done, so not valid");
            return false;
        }
        let mut set = Vec::with_capacity(10);
        for row in 0..9 {
            for elem in 0..9 {
                if let Cell::Known(val) = *myself.row(row, elem) {
                    set.push(val);
                }
            }
            set.sort();
            for elem in 0..9 {
                if set[elem] != (elem + 1) as u8 {
                    debug!("Row {} invalid for {}", row, elem);
                    return false;
                }
            }
            set.clear();
        }
        for col in 0..9 {
            // collect known values in this col
            for elem in 0..9 {
                if let Cell::Known(val) = *myself.col(col, elem) {
                    set.push(val);
                }
            }
            set.sort();
            for elem in 0..9 {
                if set[elem] != (elem + 1) as u8 {
                    debug!("Col {} invalid for {}", col, elem);
                    return false;
                }
            }
            set.clear();
        }
        for block in 0..9 {
            // collect known values in this block
            for elem in 0..9 {
                if let Cell::Known(val) = *myself.subcell(block, elem) {
                    set.push(val);
                }
            }
            set.sort();
            for elem in 0..9 {
                if set[elem] != (elem + 1) as u8 {
                    debug!("Block {} invalid for {}", block, elem);
                    debug!("set: {:?}", set);
                    return false;
                }
            }
            set.clear();
        }
        true
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

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut myself = self.clone();
        for row in 0..9 {
            for col in 0..9 {
                write!(f, "{} ", match myself.row(row, col) {
                    &mut Cell::Known(val) => format!("{}", val),
                    &mut Cell::Unknown(_) => ".".into(),
                })?
            }
            write!(f, "\n")?
        }
        Ok(())
    }
}

fn update_constraints_pointwise(puzzle: &mut Puzzle, index: usize, fill_val: u8) {
    let row: u8 = index as u8 / 9;
    let col: u8 = index as u8 % 9;
    let block = (row/3)*3 + (col/3);

    for elem in 0..9 {
        if let &mut Cell::Unknown(ref mut val) = puzzle.row(row, elem) {
            val.possibles.set(fill_val as usize, false);
        }
        if let &mut Cell::Unknown(ref mut val) = puzzle.col(col, elem) {
            val.possibles.set(fill_val as usize, false);
        }
        if let &mut Cell::Unknown(ref mut val) = puzzle.subcell(block, elem) {
            val.possibles.set(fill_val as usize, false);
        }
    }
}


fn update_constraints(puzzle: &mut Puzzle) {
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
                    val.possibles.set(*item as usize, false);
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
                    val.possibles.set(*item as usize, false);
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
                    val.possibles.set(*item as usize, false);
                }
                debug!("{:?}", val.possibles);
            }
        }
        set.clear();
    }
}

fn make_choice(puzzle: &mut Puzzle, choice: (usize, u8)) {
    puzzle.0[choice.0] = Cell::Known(choice.1);
}

fn solve_puzzle(puzzle: Puzzle) -> Puzzle {
    let mut result = puzzle.clone();
    update_constraints(&mut result);
    debug!("{}", result);
    fn inner_solve(mut puzzle: Puzzle) -> Option<Puzzle> {
        // Solve this puzzle as much as possible
        while let Some(choice) = puzzle.get_forced_choice() {
            make_choice(&mut puzzle, choice);
            update_constraints_pointwise(&mut puzzle, choice.0, choice.1);
            debug!("{}", puzzle);
        }
        // Now either pop back up the stack, or randomly guess all
        // possible choices of the most constrained single cell
        if puzzle.is_done() {
            return Some(puzzle);
        }
        // some borrowck shenanigans
        let (pivot_cell_index, opts) = {
            let (pivot_cell_index, pivot_cell) = puzzle.get_most_constrained();
            match pivot_cell {
                &mut Cell::Unknown(ref opts) => (pivot_cell_index, opts.possibles.clone()),
                _ => unreachable!(),
            }
        };
        for val in opts.ones() {
            let mut new_puzzle = puzzle.with_cell_choice(pivot_cell_index, val as u8);
            debug!("Trying pivot cell {} with val {}:", pivot_cell_index, val);
            debug!("{}", new_puzzle);
            // update_constraints(&mut new_puzzle);
            update_constraints_pointwise(&mut new_puzzle, pivot_cell_index, val as u8);
            if let Some(soln) = inner_solve(new_puzzle) {
                return Some(soln);
            }
        }
        return None;
    }
    inner_solve(result).unwrap()
}

fn main() {
    use std::time::Instant;
    let _ = env_logger::init();

    let fname = match std::env::args().skip(1).next() {
        Some(file) => file,
        None => panic!("Need a file as argument"),
    };
    let fid = std::fs::File::open(fname).expect("Not a valid filename");
    let now = Instant::now();

    let mut num_puzzles = 0;
    for line in BufReader::new(fid).lines() {
        num_puzzles += 1;
        let puzzle = parse_puzzle(&line.unwrap());
        let soln = solve_puzzle(puzzle.clone());
        assert!(soln.is_valid());
        // println!("{}\n{}", puzzle, soln);
        // println!("-------------------");
    }
    let new_now = Instant::now();
    let duration = new_now.duration_since(now);
    println!("Time for {} puzzles: {:?}, time per puzzle: {:?}",
             num_puzzles, duration, duration/num_puzzles);
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
    fn solving() {
        let _ = env_logger::init();
        let p = get_puzzle();
        let s = solve_puzzle(p);
        debug!("{}", s);
        assert!(s.is_done()); 
        assert!(s.is_valid()); 
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
        debug!("{}", p);
        assert_eq!(*p.row(0, 0), Cell::Known(9));
        assert_eq!(*p.row(0, 2), Cell::Known(4));
        assert_eq!(*p.row(1, 1), Cell::Known(5));

        assert_eq!(*p.col(0, 1), Cell::Known(2));
        assert_eq!(*p.col(2, 5), Cell::Known(1));

        assert_eq!(*p.subcell(2, 3), Cell::Known(1));
    }
}
