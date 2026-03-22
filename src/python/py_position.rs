use pyo3::prelude::*;

use crate::position::Position;

#[pyclass(name = "Position")]
#[derive(Clone, Copy, Debug)]
pub struct PyPosition {
    pub(super) pos: Position,
}

#[hotpath::measure_all]
#[pymethods]
impl PyPosition {
    #[new]
    pub fn new(col: u8, row: u8) -> Self {
        PyPosition {
            pos: Position::new(col, row),
        }
    }

    pub fn col(&self) -> u8 {
        self.pos.col
    }

    pub fn row(&self) -> u8 {
        self.pos.row
    }

    pub fn __str__(&self) -> String {
        self.pos.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Position({}, {})", self.pos.col, self.pos.row)
    }
}
