use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::encode;
use crate::r#move::{Move, MoveFlags};
use crate::position::Position;

#[pyclass(name = "Move")]
#[derive(Clone, Debug)]
pub struct PyMove {
    pub(super) move_: Move,
}

#[hotpath::measure_all]
#[pymethods]
impl PyMove {
    #[staticmethod]
    pub fn from_rowcol(src_col: usize, src_row: usize, dst_col: usize, dst_row: usize) -> Self {
        PyMove {
            move_: Move::from_position(
                Position::new(src_col, src_row),
                Position::new(dst_col, dst_row),
                MoveFlags::empty(),
            ),
        }
    }

    #[classmethod]
    pub fn from_lan(
        _cls: &Bound<'_, PyType>,
        lan: &str,
        board_width: usize,
        board_height: usize,
    ) -> PyResult<Self> {
        match Move::from_lan(lan, board_width, board_height) {
            Ok(move_) => Ok(PyMove { move_ }),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
        }
    }

    pub fn src_square(&self) -> (usize, usize) {
        (self.move_.src.col, self.move_.src.row)
    }

    pub fn dst_square(&self) -> (usize, usize) {
        (self.move_.dst.col, self.move_.dst.row)
    }

    pub fn promotion(&self) -> Option<String> {
        if self.move_.flags.contains(MoveFlags::PROMOTION) {
            if let Some(promo) = self.move_.promotion {
                Some(promo.to_char().to_string())
            } else {
                Some("q".to_string())
            }
        } else {
            None
        }
    }

    pub fn to_lan(&self) -> String {
        self.move_.to_lan()
    }

    // ---------------------------------------------------------------------
    // Encoding/decoding
    // ---------------------------------------------------------------------

    pub fn encode(&self, width: usize, height: usize) -> Option<usize> {
        encode::encode_action(&self.move_, width, height)
    }

    // ---------------------------------------------------------------------
    // Dunder Methods
    // ---------------------------------------------------------------------

    pub fn __str__(&self) -> String {
        self.move_.to_lan()
    }

    pub fn __repr__(&self) -> String {
        format!("Move({})", self.move_.to_lan())
    }

    pub fn __eq__(&self, other: &PyMove) -> bool {
        self.move_ == other.move_
    }

    pub fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.move_.src.col.hash(&mut hasher);
        self.move_.src.row.hash(&mut hasher);
        self.move_.dst.col.hash(&mut hasher);
        self.move_.dst.row.hash(&mut hasher);
        hasher.finish()
    }
}
