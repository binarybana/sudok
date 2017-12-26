#![feature(nll)]

#[macro_use]
extern crate lazy_static;

#[macro_use] extern crate log;
extern crate env_logger;

use std::io::{BufRead, BufReader};

mod bitcube;

lazy_static! {
    static ref NEIGHBORS: Vec<Vec<usize>> = {
        let mut v = Vec::with_capacity(81);
        for i in 0..81 {
            v.push(Vec::with_capacity(9+9+9-3-3));
            let row = i/9;
            let col = i%9;

            //rows
            ((row*9)..((row+1)*9)).for_each(|j| if j != i {v[i].push(j)});

            // cols
            for j in 0..9 {
                let ind = j*9 + col;
                if ind != i {
                    v[i].push(ind);
                }
            }

            // blocks
            let block_row = (row/3)*3;
            let block_col = (col/3)*3;
            for bi in 0..3 {
                if (block_row + bi) != row {
                    for bj in 0..3 {
                        if (block_col + bj) != col {
                            v[i].push((block_row+bi)*9 + (block_col+bj));
                        }
                    }
                }
            }
            assert_eq!(v[i].len(), 9+9-1+9-6);
        }
        v
    };
}

type Puzzle = bitcube::BitCube;
use bitcube::Cell;

impl Puzzle {
    fn row(&mut self, row: u8, i: u8) -> &mut Cell {
        assert!(row <9 && i <9);
        &mut self[(row*9+i) as usize]
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
        for i in 0..81 {
            if self[i].num_choices() != 1 {
                return false;
            }
        }
        true
    }

    fn get_most_constrained(&mut self) -> (usize, &mut Cell) {
        assert!(!self.is_done());
        // TODO: rewrite using functional operators
        debug!("most constrained: \n{}", self);
        let mut min_options = 11;
        let mut min_ind = 0;
        for i in 0..81 {
            let dof = self[i].num_choices();
            if dof > 1 {
                debug!("Considering index {} which has {} options", i, self[i].num_choices());
                if dof < min_options {
                        min_options = self[i].num_choices();
                        min_ind = i;
                }
            }
        }
        debug!("Just found most constrained cell at index {} with {} options", min_ind, min_options);
        (min_ind, &mut self[min_ind])
    }

    fn with_cell_choice(&self, pivot_cell_index: usize, val: u8) -> Puzzle {
        let mut new_puzzle = self.clone();
        new_puzzle[pivot_cell_index].set_value(val as usize);
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
                set.push(self[row*9 + elem].get_value());
            }
            set.sort();
            for elem in 0..9 {
                // TODO Check this u8 for off-by-one
                if set[elem] != (elem + 1) {
                    debug!("Row {} invalid for {}", row, elem);
                    return false;
                }
            }
            set.clear();
        }
        for col in 0..9 {
            // collect known values in this col
            for elem in 0..9 {
                set.push(self[elem*9 + col].get_value());
            }
            set.sort();
            for elem in 0..9 {
                // TODO Check this u8 for off-by-one
                if set[elem] != (elem + 1) {
                    debug!("Col {} invalid for {}", col, elem);
                    return false;
                }
            }
            set.clear();
        }
        for block in 0..9 {
            // collect known values in this block
            for elem in 0..9 {
                set.push(myself.subcell(block, elem).get_value());
            }
            set.sort();
            for elem in 0..9 {
                // TODO Check this u8 for off-by-one
                if set[elem] != (elem + 1) {
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
    let mut p = Puzzle::new();
    for (i, c) in flat.chars().enumerate() {
        if c != '.' {
            let val = c.to_digit(10).unwrap();
            p[i].set_value(val as usize);
        }
    }
    p
}

fn update_constraints_pointwise(puzzle: &mut Puzzle, index: usize, fill_val: u8) -> bool {
    debug!("Updating pointwise index {} value {}", index, fill_val);
    // TODO: get rid of this allocation
    for elem in NEIGHBORS[index].iter() {
        // let mut val = puzzle[*elem];
        let pre_dof = puzzle[*elem].num_choices();
        puzzle[*elem].clear(fill_val as usize);
        let dof = puzzle[*elem].num_choices();
        if dof == 0 { // Conflict
            return false;
        } else if dof == 1 && pre_dof == 2 {
            //Just clarified this choice, so go ahead and recursively
            //update_constraints_pointwise there too
            make_choice(puzzle, (*elem, puzzle[*elem].get_value() as u8));
            if !update_constraints_pointwise(puzzle, *elem, puzzle[*elem].get_value() as u8) {
                return false;
            }
        }
    }
    debug!("Pointwise assignment worked out, returning true");
    true // valid
}


fn update_constraints(puzzle: &mut Puzzle) {
    let mut set = Vec::with_capacity(10);
    for row in 0..9 {
        // collect known values in this row
        for elem in 0..9 {
            let val = puzzle[row*9+elem];
            if val.num_choices() == 1 {
                set.push(val.get_value());
            }
        }
        // Now remove that from possible set list
        for elem in 0..9 {
            for item in set.iter() {
                if puzzle[row*9+elem].num_choices() != 1 {
                    puzzle[row*9+elem].clear(*item as usize);
                }
            }
        }
        set.clear();
    }
    for col in 0..9 {
        // collect known values in this col
        for elem in 0..9 {
            let val = puzzle[elem*9+col];
            if val.num_choices() == 1 {
                set.push(val.get_value());
            }
        }
        // Now remove that from possible set list
        for elem in 0..9 {
            for item in set.iter() {
                if puzzle[elem*9+col].num_choices() != 1 {
                    puzzle[elem*9+col].clear(*item as usize);
                }
            }
        }
        set.clear();
    }
    for block in 0..9 {
        // collect known values in this block
        for elem in 0..9 {
            let val = puzzle.subcell(block,elem);
            if val.num_choices() == 1 {
                set.push(val.get_value());
            }
        }
        // Now remove that from possible set list
        for elem in 0..9 {
            for item in set.iter() {
                if puzzle.subcell(block, elem).num_choices() != 1 {
                    puzzle.subcell(block, elem).clear(*item as usize);
                }
            }
        }
        set.clear();
    }
}

fn make_choice(puzzle: &mut Puzzle, choice: (usize, u8)) {
    puzzle[choice.0].set_value(choice.1 as usize);
}

fn solve_puzzle(puzzle: Puzzle) -> Puzzle {
    let mut result = puzzle.clone();
    update_constraints(&mut result);
    debug!("\n{}", result);
    fn inner_solve(mut puzzle: Puzzle) -> Option<Puzzle> {
        // Solve this puzzle as much as possible
        // while let Some(choice) = puzzle.get_forced_choice() {
        //     make_choice(&mut puzzle, choice);
        //     update_constraints_pointwise(&mut puzzle, choice.0, choice.1);
        //     debug!("\n{}", puzzle);
        // }
        // Now either pop back up the stack, or randomly guess all
        // possible choices of the most constrained single cell
        if puzzle.is_done() {
            return Some(puzzle);
        }
        // some borrowck shenanigans
        let (pivot_cell_index, opts) = {
            let (pivot_cell_index, opts) = puzzle.get_most_constrained();
            (pivot_cell_index, opts.clone())
        };
        for val in 1..10 {
            if opts.get(val) {
                let mut new_puzzle = puzzle.with_cell_choice(pivot_cell_index, val as u8);
                if update_constraints_pointwise(&mut new_puzzle, pivot_cell_index, val as u8) {
                    if let Some(soln) = inner_solve(new_puzzle) {
                        return Some(soln);
                    }
                }
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
        let c = Cell::singleton(9);
        assert_eq!(p[0], c);
    }

    #[test]
    fn constraint_update() {
        let mut p = get_puzzle();
        update_constraints(&mut p);
    }

    #[test]
    fn neighbors() {
        assert_eq!(NEIGHBORS[0], vec![1,2,3,4,5,6,7,8,9,18,27,36,45,54,63,72,10,11,19,20]);
    }
}
