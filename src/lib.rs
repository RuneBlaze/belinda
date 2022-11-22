mod exposure;
use exposure::{set_nthreads, ClusterSkeleton, Clustering, Graph, SummarizedDistributionWrapper};
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn belinda(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Graph>()?;
    m.add_class::<ClusterSkeleton>()?;
    m.add_class::<Clustering>()?;
    m.add_class::<SummarizedDistributionWrapper>()?;
    m.add_function(wrap_pyfunction!(set_nthreads, m)?)?;
    Ok(())
}
