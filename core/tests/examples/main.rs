use assert_matches::assert_matches;
use nickel_lang_core::{
    error::{Error, EvalErrorData, EvalErrorKind},
    program::ProgramBuilder,
};
use nickel_lang_utils::{
    annotated_test::{TestCase, read_annotated_test_case},
    project_root::project_root,
    test_program::TestProgram,
};
use serde::Deserialize;
use test_generator::test_resources;

#[test_resources("examples/**/*.ncl")]
fn check_example_file(path: &str) {
    let test: TestCase<Expectation> =
        read_annotated_test_case(path).expect("Failed to parse annotated program");

    // `test_resources` uses paths relative to the workspace manifesty
    let mut p: TestProgram = ProgramBuilder::new()
        .add_path(project_root().join(path))
        .with_trace(std::io::stderr())
        .build()
        .expect("Failed to load program from file");

    match test.annotation {
        Expectation::Pass => {
            p.eval_deep()
                .expect("Example is marked as 'pass' but evaluation failed");
        }
        Expectation::Blame => assert_matches!(
            p.eval_deep(),
            Err(Error::EvalError(data)) if matches!(*data, EvalErrorData {
                error: EvalErrorKind::BlameError { .. },
                ctxt: _
            })
        ),
        Expectation::Ignore => (),
    }
}

#[derive(Deserialize)]
#[serde(tag = "test")]
enum Expectation {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "blame")]
    Blame,
    #[serde(rename = "ignore")]
    Ignore,
}
