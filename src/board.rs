use std::{iter, ops::Rem};

struct Board {
    width: usize,
    heigth: usize,
    section_size: usize,
    fields: Vec<bool>,
}

enum BoardClear {
    Row(usize),
    Column(usize),
    Section(usize),
}

enum PlaceError {
    Taken,
    OutOfBounds,
}

impl Board {
    pub fn new(width: usize, heigth: usize, section_size: usize) -> Self {
        if width == 0 || heigth == 0 || section_size == 0 {
            panic!("Invalid dimension - no dimension can be 0");
        }

        if width * heigth % (section_size * section_size) != 0 {
            panic!("Invalid section size {section_size}");
        }

        Self {
            width,
            heigth,
            section_size,
            fields: iter::repeat(false).take(width * heigth).collect(),
        }
    }

    pub fn can_place_piece(
        &self,
        x: usize,
        y: usize,
        piece: &[usize],
    ) -> Result<(), Vec<PlaceError>> {
        let mut errors = Vec::new();

        for field in piece {
            let i = field + x + y * self.width;
            if i > self.width * self.heigth {
                errors.push(PlaceError::OutOfBounds);
            } else if self.fields[i] {
                errors.push(PlaceError::Taken);
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn place_piece(
        &mut self,
        x: usize,
        y: usize,
        piece: &[usize],
    ) -> Result<Vec<BoardClear>, Vec<PlaceError>> {
        self.can_place_piece(x, y, piece)?;

        let mut cleared = Vec::new();

        for field in piece {
            let col_done = self.column_done(x);
            let row_done = self.row_done(y);
            let section_done = self.section_done(x, y);

            self.fields[field + x + y * self.width] = true;

            if !col_done && self.column_done(x) {
                cleared.push(BoardClear::Column(x));
            }

            if !row_done && self.row_done(y) {
                cleared.push(BoardClear::Row(y));
            }

            if !section_done.1 && self.section_done(x, y).1 {
                cleared.push(BoardClear::Section(section_done.0));
            }
        }

        for x in cleared.iter() {
            match x {
                BoardClear::Row(row) => self.clear_row(*row),
                BoardClear::Column(col) => self.clear_column(*col),
                BoardClear::Section(section) => self.clear_section(*section),
            };
        }

        Ok(cleared)
    }

    fn row_done(&self, row: usize) -> bool {
        let from = row * self.width;
        (from..from + self.width).all(|i| self.fields[i])
    }

    fn column_done(&self, column: usize) -> bool {
        self.fields
            .iter()
            .enumerate()
            .filter(|(i, _)| i % column == 0)
            .all(|(_, taken)| *taken)
    }

    fn section_done(&self, x: usize, y: usize) -> (usize, bool) {
        let section = self.get_section(x, y);
        (section.0, section.1.iter().all(|i| self.fields[*i]))
    }

    fn get_section(&self, x: usize, y: usize) -> (usize, Vec<usize>) {
        let mut section = Vec::with_capacity(self.section_size * self.section_size);

        let row = self.heigth.div_floor(y);
        let col = self.width.div_floor(x);

        for i in 0..self.section_size {
            section.push(row * self.width + col + i);
        }

        (row * col, section)
    }

    fn get_section_by_index(&self, index: usize) -> Vec<usize> {
        let x = index.rem_euclid(self.width) * self.section_size;
        let y = index.div_floor(self.heigth / self.section_size) * self.section_size;
        self.get_section(x, y).1
    }

    fn clear_column(&mut self, column: usize) {
        for f in self
            .fields
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| i % column == 0)
            .into_iter()
        {
            *f.1 = false;
        }
    }

    fn clear_row(&mut self, row: usize) {
        let from = row * self.width;
        for i in from..from + self.width {
            self.fields[i] = false;
        }
    }

    fn clear_section(&mut self, section: usize) {
        for i in self.get_section_by_index(section) {
            self.fields[i] = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use test_case::test_case;

    #[test_case(1, 1, 1)]
    #[test_case(3, 3, 1)]
    #[test_case(3, 3, 3)]
    #[test_case(12, 9, 3)]
    #[test_case(10, 10, 3 => panics)]
    #[test_case(0, 1, 1 => panics)]
    #[test_case(1, 0, 1 => panics)]
    #[test_case(1, 1, 0 => panics)]
    fn new(width: usize, heigth: usize, section_size: usize) {
        Board::new(width, heigth, section_size);
    }
}
