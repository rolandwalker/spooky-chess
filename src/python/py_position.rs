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
    pub fn new(col: usize, row: usize) -> Self {
        PyPosition {
            pos: Position::new(col, row),
        }
    }

    pub fn col(&self) -> usize {
        self.pos.col
    }

    pub fn row(&self) -> usize {
        self.pos.row
    }

    pub fn __str__(&self) -> String {
        self.pos.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Position({}, {})", self.pos.col, self.pos.row)
    }
}
