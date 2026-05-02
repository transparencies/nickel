use nickel_lang_core::{
    error::Sink,
    eval::cache::lazy::CBNCache,
    program::{Program, ProgramBuilder},
};

// Regression test for https://github.com/tweag/nickel/issues/2362
#[test]
fn multiple_inputs_non_paths() {
    let mut prog: Program<CBNCache> = ProgramBuilder::new()
        .add_source_string("{}", "fst")
        .add_source_string("{} & {}", "snd")
        .with_trace(std::io::stderr())
        .with_reporter(Sink::default())
        .build()
        .unwrap();

    assert_eq!(&prog.eval_full_for_export().unwrap().to_string(), "{}");
}
