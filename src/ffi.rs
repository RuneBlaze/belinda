use arrow::ffi;
use polars::frame::DataFrame;
use polars::prelude::{ArrayRef, ArrowField};
use polars::series::Series;
use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_uintptr_t;
use pyo3::prelude::*;

/// Take an arrow array from python and convert it to a rust arrow array.
/// This operation does not copy data.
fn array_to_rust(arrow_array: &PyAny) -> PyResult<ArrayRef> {
    // prepare a pointer to receive the Array struct
    let array = Box::new(ffi::ArrowArray::empty());
    let schema = Box::new(ffi::ArrowSchema::empty());

    let array_ptr = &*array as *const ffi::ArrowArray;
    let schema_ptr = &*schema as *const ffi::ArrowSchema;

    // make the conversion through PyArrow's private API
    // this changes the pointer's memory and is thus unsafe. In particular, `_export_to_c` can go out of bounds
    arrow_array.call_method1(
        "_export_to_c",
        (array_ptr as Py_uintptr_t, schema_ptr as Py_uintptr_t),
    )?;

    unsafe {
        let field = ffi::import_field_from_c(schema.as_ref()).unwrap();
        let array = ffi::import_array_from_c(*array, field.data_type).unwrap();
        Ok(array)
    }
}

/// Arrow array to Python.
pub(crate) fn to_py_array(py: Python, pyarrow: &PyModule, array: ArrayRef) -> PyResult<PyObject> {
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
    let py_array = to_py_array(py, pyarrow, series.chunks()[0].clone())?;
    let py_series = PySeries {
        name: series.name().to_string(),
        data: py_array,
    };
    Ok(py_series)
}

pub fn translate_df(df: &mut DataFrame) -> PyResult<PyObject> {
    let columns = df.get_columns_mut();
    let py_series = columns
        .iter_mut()
        .map(series_to_arrow)
        .collect::<PyResult<Vec<PySeries>>>()?;
    let gil = Python::acquire_gil();
    let py = gil.python();
    let pypolars = py.import("polars")?;
    let py_series_obj: Vec<_> = py_series
        .into_iter()
        .map(|it| {
            pypolars
                .getattr("Series")
                .unwrap()
                .call_method1("_from_arrow", (it.name, it.data))
                .unwrap()
        })
        .collect();
    let remapped_df = pypolars.getattr("DataFrame")?.call1((py_series_obj,))?;
    Ok(remapped_df.to_object(py))
}

pub fn py_series_to_rust_series(series: &PyAny) -> PyResult<Series> {
    // rechunk series so that they have a single arrow array
    let series = series.call_method0("rechunk")?;

    let name = series.getattr("name")?.extract::<String>()?;

    // retrieve pyarrow array
    let array = series.call_method0("to_arrow")?;

    // retrieve rust arrow array
    let array = array_to_rust(array)?;

    Series::try_from((name.as_str(), array)).map_err(|e| PyValueError::new_err(format!("{}", e)))
}

pub fn rust_series_to_py_series(series: &Series) -> PyResult<PyObject> {
    // ensure we have a single chunk
    let series = series.rechunk();
    let array = series.to_arrow(0);

    // acquire the gil
    Python::with_gil(|py| {
        // import pyarrow
        let pyarrow = py.import("pyarrow")?;

        // pyarrow array
        let pyarrow_array = to_py_array(py, pyarrow, array)?;

        // import polars
        let polars = py.import("polars")?;
        let out = polars.call_method1("from_arrow", (pyarrow_array,))?;
        Ok(out.to_object(py))
    })
}
