extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_smir;
extern crate stable_mir as smir;
// The rustc_smir::run macro currently expects stable_mir to be in the namespace
extern crate stable_mir;

use crate::domains::{AbstractFunction, AbstractValue, booleans, interval};
use crate::errors::*;
use crate::mir_helpers::get_fn_types;
use log::debug;
use rustc_errors::registry;
use rustc_hash::{FxHashMap, FxHashSet};
use rustc_hir::def_id::DefId;
use rustc_session::config::{self, CheckCfg};
use rustc_smir::{run, rustc_internal};
use smir::{CrateDef};
use smir::ty::{TyKind, RigidTy};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{process, str};
use crate::domains::AbstractValue::IntInterval;
use crate::domains::booleans::AbstractBool;
use crate::domains::interval::Interval;

fn get_sysroot() -> String {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();
    let sysroot = str::from_utf8(&out.stdout).unwrap().trim().to_string();
    sysroot
}

type Summary = AbstractFunction;

fn check_summaries(summaries: HashMap<String, Summary>) {
    // TODO(klinvill): placeholder
    println!("Analysis results: ");
    for (func, result) in &summaries {
        println!("{func:?}: {result:?}");
    }
}

pub fn analyze_program<B>(_tcx: rustc_middle::ty::TyCtxt) -> std::ops::ControlFlow<B> {
    let mut abstract_fns = HashMap::new();
    let all_items: Vec<smir::CrateItem> = smir::all_local_items();
    for item in all_items {
        println!("Checking function: {}", item.name());
        println!("Has kind: {:?}", item.kind());
        match analyze_function(&item.body()) {
            Ok(abstract_fn) => {
                abstract_fns.insert(item.name(), abstract_fn);
                ()
            }
            _ => (),
        };
    };

    check_summaries(abstract_fns);

    std::ops::ControlFlow::Continue(())
}

fn is_tuple(ty: &smir::ty::Ty) -> bool {
    matches!(ty.kind(), smir::ty::TyKind::RigidTy(smir::ty::RigidTy::Tuple(_)))
}

// TODO(klinvill): Can I return a reference to the params instead of the params themselves?
fn tuple_fields(ty: &smir::ty::Ty) -> Result<Vec<smir::ty::Ty>, Error> {
    match ty.kind() {
        smir::ty::TyKind::RigidTy(smir::ty::RigidTy::Tuple(params)) => Ok(params),
        _ => Err(Error::new(ErrorKind::InvalidArgumentError)),
    }
}

fn can_interpret(local_decls: &[&smir::mir::LocalDecl]) -> bool {
    fn is_numeric(ty: &smir::ty::Ty) -> bool {
        match ty.kind() {
            smir::ty::TyKind::RigidTy(smir::ty::RigidTy::Int(_)) |
            smir::ty::TyKind::RigidTy(smir::ty::RigidTy::Uint(_)) |
            smir::ty::TyKind::RigidTy(smir::ty::RigidTy::Float(_))
            => true,
            _ => false,
        }
    }

    fn is_bool(ty: &smir::ty::Ty) -> bool {
        match ty.kind() {
            smir::ty::TyKind::RigidTy(smir::ty::RigidTy::Bool) => true,
            _ => false,
        }
    }

    fn is_numeric_or_bool(ty: &smir::ty::Ty) -> bool {
        is_numeric(ty) || is_bool(ty)
    }

    local_decls.iter().map(|decl| decl.ty).all(|ty| {
        is_numeric_or_bool(&ty)
            || (is_tuple(&ty) && tuple_fields(&ty).unwrap().iter().all(|tyf| is_numeric_or_bool(&tyf)))
    })
}

fn analyze_function(function: &smir::mir::Body) -> Result<AbstractFunction, Error> {
    debug!("{function:#?}");

    let (arg_types, return_type) = get_fn_types(function);

    debug!("Argument types: {arg_types:?}");
    debug!("Return type: {return_type:?}");

    let local_decls: Vec<&smir::mir::LocalDecl> = function.locals().iter().collect();
    if can_interpret(&local_decls) {
        let abstract_fn = interpret_intervals(function);
        debug!("Abstract function: {abstract_fn:?}\n");
        let state = interpret_body(function, &vec![IntInterval(Interval::from(3))]);
        debug!("State: {state:?}\n");
        abstract_fn
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

fn interpret_body(body: &smir::mir::Body, arg_values: &Vec<AbstractValue>) -> Result<HashMap<smir::mir::Local, AbstractValue>, Error> {
    let mut state: HashMap<smir::mir::Local, AbstractValue> = HashMap::new();
    let mut errors = Vec::new();

    let (arg_types, return_type) = get_fn_types(body);
    if arg_values.len() != arg_types.len() {
        return Err(Error::with_message(
            ErrorKind::InvalidArgumentError,
            "Must supply same number of arguments as the function takes as input when interpretting it.".to_string(),
        ));
    }

    // Insert arguments into state map
    for (i, arg) in arg_values.iter().enumerate() {
        state.insert(i + 1, arg.clone());
    }

    for (bb, block) in body.blocks.iter().enumerate() {
        let result = interpret_block(block, &mut state);
        match result {
            Err(e) => errors.push(e),
            _ => (),
        }
    }
    debug!("Errors while interpreting body: {errors:#?}");
    Ok(state)
}

fn interpret_block(block: &smir::mir::BasicBlock, state: &mut HashMap<smir::mir::Local, AbstractValue>) -> Result<(), Error> {
    for statement in &block.statements {
        interpret_statement(statement, state)?;
    }
    Ok(())
}

fn interpret_statement(statement: &smir::mir::Statement, state: &mut HashMap<smir::mir::Local, AbstractValue>) -> Result<(), Error> {
    match &statement.kind {
        smir::mir::StatementKind::Assign(place, rvalue) => {
            let val = interpret_rvalue(&rvalue, state)?;
            state.insert(place.local, val);
            Ok(())
        }
        smir::mir::StatementKind::Deinit(place) => {
            state.insert(place.local, AbstractValue::Uninit);
            Ok(())
        }
        _ => Err(Error::new(ErrorKind::NotImplementedError)),
    }
}

fn interpret_rvalue(rvalue: &smir::mir::Rvalue, state: &mut HashMap<smir::mir::Local, AbstractValue>) -> Result<AbstractValue, Error> {
    match rvalue {
        smir::mir::Rvalue::Use(op) => interpret_operand(op, state),
        // TODO(klinvill): currently we assume checked operations never fail
        smir::mir::Rvalue::BinaryOp(op, left, right) => interpret_binop(op, left, right, state),
        smir::mir::Rvalue::CheckedBinaryOp(op, left, right) => {
            let v = interpret_binop(op, left, right, state)?;
            // Checked operations return the value and a boolean flag that checks if an operation succeeded
            Ok(AbstractValue::Tuple(vec![v, AbstractValue::Bool(booleans::AbstractBool::False)]))
        }
        _ => Err(Error::new(ErrorKind::NotImplementedError)),
    }
}

fn interpret_binop(binop: &smir::mir::BinOp, left: &smir::mir::Operand, right: &smir::mir::Operand, state: &mut HashMap<smir::mir::Local, AbstractValue>) -> Result<AbstractValue, Error> {
    let left_val = interpret_operand(left, state)?;
    let right_val = interpret_operand(right, state)?;
    match binop {
        smir::mir::BinOp::Add => {
            match (left_val, right_val) {
                (AbstractValue::IntInterval(l), AbstractValue::IntInterval(r)) => Ok(AbstractValue::IntInterval(l + r)),
                (AbstractValue::UintInterval(l), AbstractValue::UintInterval(r)) => Ok(AbstractValue::UintInterval(l + r)),
                _ => Err(Error::new(ErrorKind::NotImplementedError)),
            }
        }
        smir::mir::BinOp::Eq => {
            match (left_val, right_val) {
                (AbstractValue::Bool(l), AbstractValue::Bool(r)) => Ok(AbstractValue::Bool(l.equals(&r))),
                (AbstractValue::IntInterval(l), AbstractValue::IntInterval(r)) => Ok(AbstractValue::Bool(l.equals(&r))),
                (AbstractValue::UintInterval(l), AbstractValue::UintInterval(r)) => Ok(AbstractValue::Bool(l.equals(&r))),
                _ => Err(Error::new(ErrorKind::NotImplementedError)),
            }
        }
        smir::mir::BinOp::Lt => {
            match (left_val, right_val) {
                (AbstractValue::IntInterval(l), AbstractValue::IntInterval(r)) => Ok(AbstractValue::Bool(l.less_than(&r))),
                (AbstractValue::UintInterval(l), AbstractValue::UintInterval(r)) => Ok(AbstractValue::Bool(l.less_than(&r))),
                _ => Err(Error::new(ErrorKind::NotImplementedError)),
            }
        }
        _ => Err(Error::new(ErrorKind::NotImplementedError)),
    }
}

fn interpret_operand(op: &smir::mir::Operand, state: &mut HashMap<smir::mir::Local, AbstractValue>) -> Result<AbstractValue, Error> {
    match op {
        smir::mir::Operand::Copy(place) | smir::mir::Operand::Move(place) => {
            let value = get_place_value(&place, &state)?
                .ok_or(Error::new(ErrorKind::InterpreterError))?;
            // TODO(klinvill): Clone could be expensive. Should we instead wrap the abstract value
            //  in a shared reference like Rc?
            Ok(value.clone())
        }
        smir::mir::Operand::Constant(c) => match c.literal.ty().kind() {
            TyKind::RigidTy(RigidTy::Bool) => Ok(AbstractValue::Bool(booleans::AbstractBool::from(&c.literal))),
            // rustc_middle::mir::interpret::ConstValue::Scalar(s) => match s {
            //     rustc_middle::mir::interpret::Scalar::Int(i) => {
            //         let bits = i.to_bits(i.size()).unwrap();
            //         match ty.kind() {
            //             rustc_middle::ty::TyKind::Bool => {
            //                 if bits == 0 {
            //                     Ok(AbstractValue::Bool(booleans::AbstractBool::False))
            //                 } else {
            //                     Ok(AbstractValue::Bool(booleans::AbstractBool::True))
            //                 }
            //             }
            //             rustc_middle::ty::TyKind::Int(_) => {
            //                 let num = match i.size().bytes_usize() {
            //                     1 => i128::from(bits as i8),
            //                     2 => i128::from(bits as i16),
            //                     4 => i128::from(bits as i32),
            //                     8 => i128::from(bits as i64),
            //                     16 => i128::from(bits as i128),
            //                     _ => Err(Error::new(ErrorKind::NotImplementedError))?,
            //                 };
            //                 Ok(AbstractValue::IntInterval(
            //                     interval::Interval::from(num)
            //                 ))
            //             },
            //             rustc_middle::ty::TyKind::Uint(_) => {
            //                 let num = match i.size().bytes_usize() {
            //                     1 => u128::from(bits as u8),
            //                     2 => u128::from(bits as u16),
            //                     4 => u128::from(bits as u32),
            //                     8 => u128::from(bits as u64),
            //                     16 => u128::from(bits as u128),
            //                     _ => Err(Error::new(ErrorKind::NotImplementedError))?,
            //                 };
            //                 Ok(AbstractValue::UintInterval(
            //                     interval::Interval::from(num)
            //                 ))
            //             },
            //             _ => Err(Error::new(ErrorKind::NotImplementedError)),
            //         }
            //     },
            //     _ => Err(Error::new(ErrorKind::NotImplementedError)),
            // },
            //     _ => Err(Error::new(ErrorKind::NotImplementedError)),
            // },
            _ => Err(Error::new(ErrorKind::NotImplementedError)),
        }
    }
}


fn get_place_value(place: &smir::mir::Place, state: &HashMap<smir::mir::Local, AbstractValue>) -> Result<Option<AbstractValue>, Error> {
    Ok(state.get(&place.local).cloned())
}

// fn follow_projection<'a, V: std::fmt::Debug, T: std::fmt::Debug>(val: &'a AbstractValue, proj: &smir::mir::ProjectionElem<V,T>) -> Result<Option<&'a AbstractValue>, Error> {
//     match proj {
//         smir::mir::ProjectionElem::Field(f, ty) => {
//             Ok(val.get(f.index()))
//         },
//         _ => Err(Error::with_message(
//             ErrorKind::NotImplementedError,
//             format!("Projection handling is not implemented for projection {:?}", proj),
//         )),
//     }
// }
