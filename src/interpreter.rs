extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_smir as smir;

use crate::domains::{AbstractFunction, AbstractValue};
use crate::errors::*;
use crate::mir_helpers::get_fn_types;
use log::debug;
use rustc_errors::registry;
use rustc_hash::{FxHashMap, FxHashSet};
use rustc_hir::def_id::DefId;
use rustc_session::config::{self, CheckCfg};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{process, str};

fn get_sysroot() -> String {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();
    let sysroot = str::from_utf8(&out.stdout).unwrap().trim().to_string();
    sysroot
}

pub fn analyze_program(program: PathBuf) -> HashMap<DefId, AbstractFunction> {
    let sysroot = get_sysroot();
    let config = rustc_interface::Config {
        // Command line options
        opts: config::Options {
            // It's important to include the sysroot here. Otherwise I ran into errors stating the
            // std crate could not be found.
            maybe_sysroot: Some(PathBuf::from(sysroot)),
            ..config::Options::default()
        },
        crate_cfg: FxHashSet::default(),
        crate_check_cfg: CheckCfg::default(),
        input: rustc_session::config::Input::File(program),
        input_path: None,
        output_dir: None,
        output_file: None,
        file_loader: None,
        lint_caps: FxHashMap::default(),
        parse_sess_created: None,
        register_lints: None,
        override_queries: None,
        // Registry of diagnostics codes.
        registry: registry::Registry::new(rustc_error_codes::DIAGNOSTICS),
        make_codegen_backend: None,
    };

    // We need to run the compiler ourselves in order to get the MIR representation of a program.
    // Example of running the compiler through rustc_interface up here:
    //  https://github.com/rust-lang/rustc-dev-guide/blob/master/examples/rustc-driver-example.rs
    let results = rustc_interface::run_compiler(config, |compiler| {
        compiler.enter(|queries| {
            queries.global_ctxt().unwrap().take().enter(|tcx| {
                let mut abstract_fns = HashMap::new();
                // Get the optimized mir
                let keys = tcx.mir_keys(());
                debug!("All found keys in MIR: {keys:?}");
                for key in keys.iter() {
                    debug!("Checking for key: {key:?}");
                    let mir = tcx.optimized_mir(*key);
                    let abstract_result = analyze_function(mir);
                    match abstract_result {
                        Ok(abstract_fn) => {
                            abstract_fns.insert(key.to_def_id(), abstract_fn);
                            ()
                        },
                        _ => (),
                    };
                }

                abstract_fns
            })
        })
    });

    results
}

// TODO(klinvill): Should replace ty type with &smir::ty::Ty, but it looks like the latest
//  change including ty in smir hasn't been synced back to rust yet.
fn is_tuple(ty: &rustc_middle::ty::Ty) -> bool {
    // TODO(klinvill): Should replace with smir::ty::TyKind::Tuple(_), but it looks like the
    //  latest change including ty in smir hasn't been synced back to rust yet.
    matches!(ty.kind(), rustc_middle::ty::TyKind::Tuple(_))
}

fn can_interpret(local_decls: &[&smir::mir::LocalDecl]) -> bool {
    // TODO(klinvill): Should replace ty type with &smir::ty::Ty, but it looks like the latest
    //  change including ty in smir hasn't been synced back to rust yet.
    fn is_numeric_or_bool(ty: &rustc_middle::ty::Ty) -> bool {
        ty.is_numeric() || ty.is_bool()
    }

    local_decls.iter().map(|decl| decl.ty).all(|ty| {
        is_numeric_or_bool(&ty)
            || (is_tuple(&ty) && ty.tuple_fields().iter().all(|tyf| is_numeric_or_bool(&tyf)))
    })
}

fn analyze_function(function: &smir::mir::Body) -> Result<AbstractFunction, Error> {
    debug!("{function:?}");

    let (arg_types, return_type) = get_fn_types(function);

    debug!("Argument types: {arg_types:?}");
    debug!("Return type: {return_type:?}");

    let local_decls: Vec<&smir::mir::LocalDecl> = function.local_decls.iter().collect();
    if can_interpret(&local_decls) {
        let function = interpret_intervals(function);
        debug!("Abstract function: {function:?}\n");
        function
    } else {
        debug!("\n");
        Err(Error::new(ErrorKind::InterpreterError))
    }
}

fn interpret_intervals(function: &smir::mir::Body) -> Result<AbstractFunction, Error> {
    let (arg_types, return_type) = get_fn_types(function);

    // TODO(klinvill): We only keep the first error here. Should we instead be keeping track of all errors?
    let abstract_args: Result<Vec<_>, _> = arg_types.iter().map(AbstractValue::new).collect();
    let abstract_return = AbstractValue::new(&return_type);

    match (abstract_args, abstract_return) {
        (Ok(arguments), Ok(return_val)) => Ok(AbstractFunction {
            arguments,
            return_val,
        }),
        (Err(e), _) => Err(e),
        (_, Err(e)) => Err(e),
    }
}
