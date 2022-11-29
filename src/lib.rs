mod df;
mod exposure;
mod ffi;
mod io;
use exposure::{
    py_bitmap_union, py_from_assignments, py_label_cc, py_label_cc_size, py_nodeset_to_list,
    py_popcnt, py_read_assignment_file, py_read_json, set_nthreads, ClusterSkeleton, Clustering,
    Graph, SingletonMode, SummarizedDistributionWrapper,
};
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn belinda(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Graph>()?;
    m.add_class::<SingletonMode>()?;
    m.add_class::<SummarizedDistributionWrapper>()?;
    m.add_function(wrap_pyfunction!(set_nthreads, m)?)?;
    m.add_function(wrap_pyfunction!(py_popcnt, m)?)?;
    m.add_function(wrap_pyfunction!(py_bitmap_union, m)?)?;
    m.add_function(wrap_pyfunction!(py_from_assignments, m)?)?;
    m.add_function(wrap_pyfunction!(py_read_json, m)?)?;
    m.add_function(wrap_pyfunction!(py_read_assignment_file, m)?)?;
    m.add_function(wrap_pyfunction!(py_label_cc, m)?)?;
    m.add_function(wrap_pyfunction!(py_label_cc_size, m)?)?;
    m.add_function(wrap_pyfunction!(py_nodeset_to_list, m)?)?;
    Ok(())
}
