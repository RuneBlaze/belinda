mod df;
mod exposure;
mod ffi;
mod io;
use exposure::{
    py_bitmap_union, py_from_assignments, py_popcnt, set_nthreads, ClusterSkeleton,
    Clustering, Graph, SingletonMode, SummarizedDistributionWrapper, py_read_json, py_read_assignment_file,
};
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn belinda(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Graph>()?;
    m.add_class::<ClusterSkeleton>()?;
    m.add_class::<Clustering>()?;
    m.add_class::<SummarizedDistributionWrapper>()?;
    m.add_function(wrap_pyfunction!(set_nthreads, m)?)?;
    // m.add_function(wrap_pyfunction!(read_clusters, m)?)?;
    m.add_function(wrap_pyfunction!(py_popcnt, m)?)?;
    m.add_function(wrap_pyfunction!(py_bitmap_union, m)?)?;
    m.add_class::<SingletonMode>()?;
    m.add_function(wrap_pyfunction!(py_from_assignments, m)?)?;
    m.add_function(wrap_pyfunction!(py_read_json, m)?)?;
    m.add_function(wrap_pyfunction!(py_read_assignment_file, m)?)?;
    // let child_module = PyModule::new(_py, "belinda.stats")?;
    // m.add_function(wrap_pyfunction!(cpm, m)?)?;
    // m.add_submodule(child_module)?;
    Ok(())
}
