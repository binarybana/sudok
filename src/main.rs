use std::io::{BufRead, BufReader};


#[derive(Debug,Clone)]
struct GuessCell {
    possibles: Vec<u8>,
}

impl GuessCell {
    fn new() -> GuessCell {
        let mut set = Vec::with_capacity(10);
        for i in 0..10 {
            set.push(i);
        }
        GuessCell{possibles: set}
    }
}

#[derive(Debug,Clone)]
enum Cell {
    Unknown(GuessCell),
    Known(u8),
}

#[derive(Debug,Clone)]
struct Puzzle(Vec<Cell>);

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
    puzzle.0[1] = Cell::Known(3);
    for (i,el) in puzzle.0.iter().enumerate() {
        // Insert row, column and subcell logic here
        println!("{}, {:?}", i, el);
    }
}

fn solve_puzzle(puzzle: Puzzle) -> Puzzle {
    let mut result = puzzle.clone();
    update_constraints(&mut result);
    result
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

    #[test]
    fn parse_test() {
        let s = "9.4..5...25.6..1..31......8.7...9...4..26......147....7.......2...3..8.6.4.....9.";
        let p = parse_puzzle(s);
        assert_eq!(p.0[0], Some(9));
        assert_eq!(p.0[1], None);
        assert_eq!(p.0[2], Some(4));
    }
}

