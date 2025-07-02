mod distance;
mod minhash;
use pyo3::prelude::*;

use crate::distance::levenshtein;

#[pymodule]
fn _string_metrics(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein, m)?)?;
    m.add_class::<minhash::MinHash>()?;
    Ok(())
}
