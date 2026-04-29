use std::ffi::OsString;

use nickel_lang_core::{
    error::{
        Error,
        report::{ColorOpt, report_as_str},
    },
    eval::cache::{Cache, CacheImpl},
    program::{Program, ProgramBuilder},
    serialize,
};

use pyo3::{create_exception, exceptions::PyException, prelude::*};

create_exception!(nickel, NickelException, PyException);

/// Turn an internal Nickel error into a PyErr with a fancy diagnostic message
fn error_to_exception<E: Into<Error>, EC: Cache>(error: E, program: &mut Program<EC>) -> PyErr {
    NickelException::new_err(report_as_str(
        &mut program.files(),
        error.into(),
        ColorOpt::default(),
    ))
}

/// Evaluate from a Python str of a Nickel expression to a Python str of the resulting JSON.
///
/// # Parameters
///
/// - `expr`: the Nickel expression to evaluate.
/// - `import_paths`: optional list of paths to search for imported files. In the Nickel
///   stand-alone binary, the import paths are controlled by the `NICKEL_IMPORT_PATH` environment
///   variable and the `--import-path` CLI argument. In the Python bindings, you need to provide
///   them explicitly instead.
#[pyfunction]
#[pyo3(signature = (expr, import_paths=None))]
pub fn run(expr: String, import_paths: Option<Vec<OsString>>) -> PyResult<String> {
    let mut builder = ProgramBuilder::new().add_source_string(expr, "python");
    if let Some(import_paths) = import_paths {
        builder = builder.add_import_paths(import_paths);
    }
    let mut program: Program<CacheImpl> = builder
        .build()
        .expect("building from a single in-memory source cannot fail");

    let term = program
        .eval_full()
        .map_err(|error| error_to_exception(error, &mut program))?;

    serialize::validate(serialize::ExportFormat::Json, &term).map_err(|error| {
        error_to_exception(
            error.with_pos_table(program.pos_table().clone()),
            &mut program,
        )
    })?;

    let json_string =
        serialize::to_string(serialize::ExportFormat::Json, &term).map_err(|error| {
            error_to_exception(
                error.with_pos_table(program.pos_table().clone()),
                &mut program,
            )
        })?;

    Ok(json_string)
}

#[pymodule]
mod nickel {
    #[pymodule_export]
    use super::run;

    #[pymodule_export]
    use super::NickelException;
}
