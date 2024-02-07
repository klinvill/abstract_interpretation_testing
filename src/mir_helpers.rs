extern crate stable_mir as smir;

// TODO(klinvill): Would be more efficient to just return references to the types along with a
//  lifetime annotation matching that of the `function` argument (instead of implicitly making
//  copies of the types).
pub(crate) fn get_fn_types(function: &smir::mir::Body) -> (Vec<smir::ty::Ty>, smir::ty::Ty) {
    // Currently it looks like the first local in Body is always the return local while the argument
    // locals always immediately follow.
    let return_type = function.ret_local().ty;

    // TODO(klinvill): How can we get the number of input arguments to the function? The function's
    //  type should have a GenericArgs argument that could be used, but how do we get the functions
    //  type/definition?
    log::warn!("Cannot currently reliably extract the number of input arguments to a function");
    let arg_types = function.arg_locals().iter().map(|l| l.ty).collect();
    return (arg_types, return_type)
}


#[cfg(test)]
mod tests {
    use crate::test_utils;

    mod get_fn_types_tests {
        use crate::mir_helpers::get_fn_types;
        use crate::test_utils;

        extern crate rustc_middle;
        extern crate stable_mir as smir;
        use rustc_middle::ty::TyCtxt;
        use smir::ty::{TyKind, RigidTy, IntTy, UintTy};
        use smir::{CrateDef};

        // Helper function that returns if t matches the unit return type. The unit return type is
        // currently represented as a tuple with no entries.
        fn matches_unit(t: &TyKind) -> bool {
            match t {
                // I don't believe we can match against an empty vector without matching on a slice.
                // We therefore use nested match statements to match against the vec in the Tuple
                // kind as a slice.
                TyKind::RigidTy(RigidTy::Tuple(inner)) => match inner[..] {
                    [] => true,
                    _ => false,
                },
                _ => false,
            }
        }

        #[test]
        fn function_with_no_arguments() {
            let body = r"fn no_args() { }".to_string();
            fn callback (tcx: TyCtxt<'_>) -> std::ops::ControlFlow<()> {
                let items = smir::all_local_items();
                assert_eq!(items.len(), 1);

                let (args_type, return_type) = get_fn_types(&items[0].body());
                // Note(klinvill): the Stable MIR Ty type doesn't support equality, so instead we match against the expected result.
                // assert_eq!(args_type, expected_args_type);
                // assert_eq!(return_type, expected_return_type);

                // Note(klinvill): due to how matching works, we need to avoid matching against
                //  variables and instead match against values. Matching against variables will
                //  simply bind any value to the variable and therefore trivially match. Macros can
                //  be used in place of variables if needed.
                assert!(matches!(args_type[..], []));
                assert!(matches_unit(&return_type.kind()));

                // If we don't continue, the execution process will be interrupted and the test will fail.
                std::ops::ControlFlow::Continue(())
            };

            test_utils::mir_test(body, callback)
        }

        #[test]
        fn function_with_no_arguments_int_return() {
            let body = r"fn no_args() -> i32 { 42 }".to_string();
            fn callback (tcx: TyCtxt<'_>) -> std::ops::ControlFlow<()> {
                let items = smir::all_local_items();
                assert_eq!(items.len(), 1);

                // Note(klinvill): due to how matching works, we need to avoid matching against
                //  variables and instead match against values. Matching against variables will
                //  simply bind any value to the variable and therefore trivially match. Macros can
                //  be used in place of variables if needed.
                let (args_type, return_type) = get_fn_types(&items[0].body());
                assert!(matches!(args_type[..], []));
                assert!(matches!(return_type.kind(), TyKind::RigidTy(RigidTy::Int(IntTy::I32))));

                // If we don't continue, the execution process will be interrupted and the test will fail.
                std::ops::ControlFlow::Continue(())
            };

            test_utils::mir_test(body, callback)
        }

        #[test]
        fn function_with_int_argument_no_return() {
            let body = r"fn no_return(x: i32) { if x == 5 {()} else {()} }".to_string();
            fn callback (tcx: TyCtxt<'_>) -> std::ops::ControlFlow<()> {
                let items = smir::all_local_items();
                assert_eq!(items.len(), 1);
                println!("Statement kinds: {:?}", &items[0].kind());
                println!("Statement name: {:?}", &items[0].name());
                println!("Statement span: {:?}", &items[0].span());
                println!("Statement body: {:?}", &items[0].body());

                // Note(klinvill): due to how matching works, we need to avoid matching against
                //  variables and instead match against values. Matching against variables will
                //  simply bind any value to the variable and therefore trivially match. Macros can
                //  be used in place of variables if needed.
                let (args_type, return_type) = get_fn_types(&items[0].body());
                let arg_kinds: Vec<_> = args_type.iter().map(|arg| arg.kind()).collect();
                println!("arg_kinds is: {:?}", arg_kinds);
                println!("return_type is: {:?}", return_type);
                assert!(matches!(arg_kinds[..], [TyKind::RigidTy(RigidTy::Int(IntTy::I32))]));
                assert!(matches_unit(&return_type.kind()));

                // If we don't continue, the execution process will be interrupted and the test will fail.
                std::ops::ControlFlow::Continue(())
            };

            test_utils::mir_test(body, callback)
        }

        #[test]
        fn function_with_bool_argument_int_return() {
            let body = r"fn test(b: bool) -> i32 { if b {5} else {7} }".to_string();
            fn callback (tcx: TyCtxt<'_>) -> std::ops::ControlFlow<()> {
                let items = smir::all_local_items();
                assert_eq!(items.len(), 1);

                // Note(klinvill): due to how matching works, we need to avoid matching against
                //  variables and instead match against values. Matching against variables will
                //  simply bind any value to the variable and therefore trivially match. Macros can
                //  be used in place of variables if needed.
                let (args_type, return_type) = get_fn_types(&items[0].body());
                let arg_kinds: Vec<_> = args_type.iter().map(|arg| arg.kind()).collect();
                println!("arg_kinds is: {:?}", arg_kinds);
                println!("return_type is: {:?}", return_type);
                assert!(matches!(arg_kinds[..], [TyKind::RigidTy(RigidTy::Bool)]));
                assert!(matches!(return_type.kind(), TyKind::RigidTy(RigidTy::Int(IntTy::I32))));

                // If we don't continue, the execution process will be interrupted and the test will fail.
                std::ops::ControlFlow::Continue(())
            };

            test_utils::mir_test(body, callback)
        }

        #[test]
        fn function_with_multiple_arguments_uint_return() {
            let body = r"fn test(b: bool, x: i32) -> u32 { if b && x > 0 {x as u32} else {7} }".to_string();
            fn callback (_: TyCtxt<'_>) -> std::ops::ControlFlow<()> {
                let items = smir::all_local_items();
                assert_eq!(items.len(), 1);

                println!("Statement kinds: {:?}", &items[0].kind());
                println!("Statement name: {:?}", &items[0].name());
                println!("Statement span: {:?}", &items[0].span());
                println!("Statement body: {:#?}", &items[0].body());

                // Note(klinvill): due to how matching works, we need to avoid matching against
                //  variables and instead match against values. Matching against variables will
                //  simply bind any value to the variable and therefore trivially match. Macros can
                //  be used in place of variables if needed.
                let (args_type, return_type) = get_fn_types(&items[0].body());
                let arg_kinds: Vec<_> = args_type.iter().map(|arg| arg.kind()).collect();
                println!("arg_kinds is: {:?}", arg_kinds);
                println!("return_type is: {:?}", return_type);
                assert!(matches!(arg_kinds[..], [
                    TyKind::RigidTy(RigidTy::Bool),
                    TyKind::RigidTy(RigidTy::Int(IntTy::I32)),
                ]));
                assert!(matches!(return_type.kind(), TyKind::RigidTy(RigidTy::Uint(UintTy::U32))));

                // If we don't continue, the execution process will be interrupted and the test will fail.
                std::ops::ControlFlow::Continue(())
            };

            test_utils::mir_test(body, callback)
        }
    }
}
