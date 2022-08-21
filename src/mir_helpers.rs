extern crate rustc_middle;
extern crate rustc_smir as smir;

// TODO(klinvill): Should replace ty type with &smir::ty::Ty, but it looks like the latest
//  change including ty in smir hasn't been synced back to rust yet.
// TODO(klinvill): Would be more efficient to just return references to the types along with a
//  lifetime annotation matching that of the `function` argument (instead of implicitly making
//  copies of the types).
pub(crate) fn get_fn_types<'tcx>(function: &smir::mir::Body<'tcx>) -> (Vec<rustc_middle::ty::Ty<'tcx>>, rustc_middle::ty::Ty<'tcx>) {
    // Currently it looks like the first declaration in Body is always the return type while the
    // argument types always immediately follow.
    let arg_types: Vec<_> = function
        .local_decls
        .iter()
        .skip(1)
        .take(function.arg_count)
        .map(|decl| decl.ty)
        .collect();
    let return_type = function.local_decls.iter().next().unwrap().ty;

    (arg_types, return_type)
}
