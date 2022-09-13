use std::{iter, ops::Rem};

struct Board<const S: usize, const W: usize, const H: usize = W> {
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

impl<const S: usize, const W: usize, const H: usize> Board<S, W, H> {
    pub fn new() -> Self {
        if W * H % (S * S) != 0 {
            panic!("Invalid section size {}", S);
        }

        Self {
            fields: iter::repeat(false).take(W * H).collect(),
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
            let i = field + x + y * W;
            if i > W * H {
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

            self.fields[field + x + y * W] = true;

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

    fn get_change_count(&self, x: usize, y: usize) -> usize {
        [].iter().filter(|taken| **taken).count()
    }

    fn row_done(&self, row: usize) -> bool {
        let from = row * W;
        (from..from + W).all(|i| self.fields[i])
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
        let mut section = Vec::with_capacity(S * S);

        let row = H.div_floor(y);
        let col = W.div_floor(x);

        for i in 0..S {
            section.push(row * W + col + i);
        }

        (row * col, section)
    }

    fn get_section_by_index(&self, index: usize) -> Vec<usize> {
        let x = index.rem_euclid(W) * S;
        let y = index.div_floor(H / S) * S;
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
        let from = row * W;
        for i in from..from + W {
            self.fields[i] = false;
        }
    }

    fn clear_section(&mut self, section: usize) {
        for i in self.get_section_by_index(section) {
            self.fields[i] = false;
        }
    }
}
