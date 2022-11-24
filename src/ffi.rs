use polars::frame::DataFrame;
use polars::series::Series;
use polars::prelude::{ArrayRef, ArrowField};
use pyo3::ffi::Py_uintptr_t;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use arrow::ffi;

/// Arrow array to Python.
pub(crate) fn to_py_array(array: ArrayRef, py: Python, pyarrow: &PyModule) -> PyResult<PyObject> {
    let schema = Box::new(ffi::export_field_to_c(&ArrowField::new(
        "",
        array.data_type().clone(),
        true,
    )));
    let array = Box::new(ffi::export_array_to_c(array));

    let schema_ptr: *const ffi::ArrowSchema = &*schema;
    let array_ptr: *const ffi::ArrowArray = &*array;

    let array = pyarrow.getattr("Array")?.call_method1(
        "_import_from_c",
        (array_ptr as Py_uintptr_t, schema_ptr as Py_uintptr_t),
    )?;

    Ok(array.to_object(py))
}

#[pyclass]
pub struct PySeries { 
    #[pyo3(get, set)]
    name: String, 
    #[pyo3(get, set)]
    data: PyObject,
}

pub fn series_to_arrow(series: &mut Series) -> PyResult<PySeries> {
    let series = series.rechunk();
    let gil = Python::acquire_gil();
    let py = gil.python();
    let pyarrow = py.import("pyarrow")?;
    let py_array = to_py_array(series.chunks()[0].clone(), py, pyarrow)?;
    let py_series = PySeries{
        name: series.name().to_string(),
        data: py_array
    };
    Ok(py_series)
}