use crate::r#type::{Ty, Universe};

pub fn type_universe(ty: &Ty) -> Universe {
    match ty {
        Ty::Unit => Universe::FreeUniverse,
        Ty::Boolean => Universe::FreeUniverse,
        Ty::Integer(_, _) => Universe::FreeUniverse,
        Ty::SingleFloat => Universe::FreeUniverse,
        Ty::DoubleFloat => Universe::FreeUniverse,
        //Ty::NamedType(_, _, u) => u,
        //Ty::RegionTy(_) => Universe::RegionUniverse,
        Ty::ReadRef(_, _) => Universe::FreeUniverse,
        Ty::WriteRef(_, _) => Universe::LinearUniverse,
        Ty::Span(_, _) => Universe::FreeUniverse,
        Ty::SpanMut(_, _) => Universe::LinearUniverse,
        //Ty::TyVar(TypeVariable(_, u, _, _)) => u,
        Ty::Address(_) => Universe::FreeUniverse,
        Ty::Pointer(_) => Universe::FreeUniverse,
        Ty::FnPtr(_, _) => Universe::FreeUniverse,
        //Ty::MonoTy(_) => unreachable!("You shouldn't be asking for the type_universe of a MonoTy"),
    }
}

pub fn is_numeric(ty: &Ty) -> bool {
    match ty {
        Ty::Unit => false,
        Ty::Boolean => false,
        Ty::Integer(_, _) => true,
        Ty::SingleFloat => true,
        Ty::DoubleFloat => true,
        //Ty::NamedType(_, _, _) => false,
        //Ty::RegionTy(_) => false,
        Ty::ReadRef(_, _) => false,
        Ty::WriteRef(_, _) => false,
        Ty::Span(_, _) => false,
        Ty::SpanMut(_, _) => false,
        //Ty::TyVar(_) => false,
        Ty::Address(_) => false,
        Ty::Pointer(_) => false,
        Ty::FnPtr(_, _) => false,
        //Ty::MonoTy(_) => unreachable!("You shouldn't be asking for the is_numeric of a MonoTy"),
    }
}

pub fn is_integer(ty: &Ty) -> bool {
    match ty {
        Ty::Unit => false,
        Ty::Boolean => false,
        Ty::Integer(_, _) => true,
        Ty::SingleFloat => false,
        Ty::DoubleFloat => false,
        //Ty::NamedType(_, _, _) => false,
        //Ty::RegionTy(_) => false,
        Ty::ReadRef(_, _) => false,
        Ty::WriteRef(_, _) => false,
        Ty::Span(_, _) => false,
        Ty::SpanMut(_, _) => false,
        //Ty::TyVar(_) => false,
        Ty::Address(_) => false,
        Ty::Pointer(_) => false,
        Ty::FnPtr(_, _) => false,
        //Ty::MonoTy(_) => unreachable!("You shouldn't be asking for the is_integer of a MonoTy"),
    }
}
