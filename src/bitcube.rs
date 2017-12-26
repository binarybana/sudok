use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell(u16);

impl Cell {
    pub fn singleton(val: usize) -> Cell {
        Cell{ 0: 1<<val}
    }

    pub fn get(&self, index: usize) -> bool {
        (self.0 & (1 << index)) > 0
    }

    pub fn set(&mut self, index: usize) {
        assert!(index < 16);
        self.0 = self.0 | (1 << index)
    }

    pub fn clear(&mut self, index: usize) {
        assert!(index < 16);
        self.0 = self.0 & !(1 << index)
    }

    pub fn num_choices(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn get_value(&self) -> usize {
        assert_eq!(self.num_choices(), 1);
        self.0.trailing_zeros() as usize
    }

    pub fn set_value(&mut self, index: usize) {
        self.0 = 1 << index;
    }
}


#[derive(Clone)]
pub struct BitCube {
    storage: [Cell; 81],
}

impl ::std::fmt::Display for BitCube {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        for row in 0..9 {
            for col in 0..9 {
                write!(f, "{} ", match self[row*9 + col] {
                    v if v.num_choices() == 1 => format!("{}", v.get_value()),
                    _ => ".".into(),
                })?
            }
            write!(f, "\n")?
        }
        Ok(())
    }
}

impl BitCube {
    pub fn new() -> BitCube {
        let nine_set = 0b1111111110;
        BitCube{ storage: [Cell{0: nine_set}; 81] }
    }
}

impl Index<usize> for BitCube {
    type Output = Cell;
    fn index(&self, index: usize) -> &Self::Output {
        &self.storage[index]
    }

}

impl IndexMut<usize> for BitCube {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.storage[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell() {
        let mut c = Cell{0: 0};
        c.set(2);
        assert_eq!(c.0, 4);
        assert_eq!(c.num_choices(), 1);
        assert_eq!(c.get_value(), 2);

        c.set(4);
        assert_eq!(c.0, 20);
        assert_eq!(c.num_choices(), 2);

        c.set(1);
        assert_eq!(c.0, 22);
        assert_eq!(c.num_choices(), 3);

        c.clear(1);
        assert_eq!(c.num_choices(), 2);
        c.clear(2);
        assert_eq!(c.0, 16);
        assert_eq!(c.num_choices(), 1);
        assert_eq!(c.get_value(), 4);

        c.clear(4);
        assert_eq!(c.0, 0);
        assert_eq!(c.num_choices(), 0);

        c.set(1);
        assert_eq!(c.0, 2);
        assert_eq!(c.num_choices(), 1);
        c.clear(2);
        assert_eq!(c.0, 2);
        assert_eq!(c.num_choices(), 1);
        c.clear(2);
        assert_eq!(c.0, 2);
        assert_eq!(c.num_choices(), 1);
    }

    #[test]
    fn bitcube() {
        let mut b = BitCube{ storage: [Cell{0:0}; 81] };
        b[0].set(2);
        b[1].set(4);

        assert_eq!(b[0].0, 4);
        assert_eq!(b[1].0, 16);
        assert_eq!(b[2].0, 0);

        let b = BitCube::new();
        assert_eq!(b[0].num_choices(), 9);
    }
}
