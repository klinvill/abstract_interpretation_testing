#![feature(rustc_private)]
#![feature(box_patterns)]

pub mod domains;
pub mod errors;
pub mod interpreter;
mod mir_helpers;

#[macro_use]
extern crate rustc_smir;

#[cfg(test)]
pub(crate) mod test_utils {
    extern crate rustc_driver;
    extern crate rustc_interface;
    extern crate rustc_middle;
    extern crate stable_mir;
    use std::ops::ControlFlow;
    use rustc_middle::ty::TyCtxt;
    use rustc_smir::{run_with_tcx, rustc_internal};
    use std::io::Write;
    use stable_mir::ty::{TyKind, RigidTy, IntTy, UintTy};

    pub(crate) fn tmp_program(program_body: String) -> tempfile::NamedTempFile {
        let mut f = tempfile::Builder::new()
            .prefix("tmp")
            .suffix(".rs")
            .tempfile()
            .unwrap();
        f.write_all(program_body.as_bytes()).unwrap();
        f
    }

    pub(crate) fn mir_test(program_body: String, callback: fn(TyCtxt<'_>) -> std::ops::ControlFlow<()>) {
        let program = tmp_program(program_body);

        let rustc_args = vec![
            "--smir-check".into(),
            // Lib crates don't require main functions
            "--crate-type=lib".into(),
            program.path().to_path_buf().into_os_string().into_string().unwrap(),
        ];
        run_with_tcx!(rustc_args, callback).unwrap();
    }
}
