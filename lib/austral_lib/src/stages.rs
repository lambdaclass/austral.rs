use crate::{
    common::{ComparisonOperator, DeclId, Identifier, Mutability, QIdent},
    escape::EscapedString,
    r#type::{Ty, ValueParameter},
    span::Span,
};

pub enum TStmt {
    TSkip(Span),
    TLet(Span, Identifier, Box<TExpr>, Mutability, Ty, Box<TStmt>),
    TDestructure(Span, Mutability, Vec<TypedBinding>, Box<TExpr>, Box<TStmt>),
    TAssign(Span, Box<TExpr>, Box<TExpr>),
    TAssignVar(Span, QIdent, Box<TExpr>),
    TInitialAssign(QIdent, Box<TExpr>),
    TIf(Span, Box<TExpr>, Box<TStmt>, Box<TStmt>),
    TCase(Span, Box<TExpr>, Vec<TypedWhen>, CaseRef),
    TWhile(Span, Box<TExpr>, Box<TStmt>),
    TFor(Span, Identifier, Box<TExpr>, Box<TExpr>, Box<TStmt>),
    TBorrow {
        span: Span,
        original: Identifier,
        rename: Identifier,
        region: Identifier,
        orig_type: Ty,
        ref_type: Ty,
        body: Box<TStmt>,
        mode: BorrowStmtKind,
    },
    TBlock(Span, Box<TStmt>, Box<TStmt>),
    TDiscarding(Span, Box<TExpr>),
    TReturn(Span, Box<TExpr>),
    TLetTmp(Identifier, Ty, Box<TExpr>),
    TAssignTmp(Identifier, Box<TExpr>),
}

pub enum TExpr {
    TNilConstant,
    TBoolConstant(bool),
    TIntConstant(String),
    TFloatConstant(String),
    TStringConstant(EscapedString),
    TConstVar(QIdent, Ty),
    TParamVar(Identifier, Ty),
    TLocalVar(Identifier, Ty),
    //TFunVar(DeclId, Ty, TypeBindings),
    TTemporary(Identifier, Ty),
    //TFuncall(DeclId, QIdent, Vec<Box<TExpr>>, Ty, TypeBindings),
    //TMethodCall(InsMethId, QIdent, TyParams, Vec<Box<TExpr>>, Ty, TypeBindings),
    TVarMethodCall {
        source_module_name: ModuleName,
        typeclass_id: DeclId,
        params: Vec<ValueParameter>,
        method_name: QIdent,
        args: Vec<Box<TExpr>>,
        dispatch_ty: Ty,
        rt: Ty,
        //bindings: TypeBindings,
    },
    TFptrCall(Identifier, Vec<Box<TExpr>>, Ty),
    TCast(Box<TExpr>, Ty),
    TComparison(ComparisonOperator, Box<TExpr>, Box<TExpr>),
    TConjunction(Box<TExpr>, Box<TExpr>),
    TDisjunction(Box<TExpr>, Box<TExpr>),
    TNegation(Box<TExpr>),
    TIfExpression(Box<TExpr>, Box<TExpr>, Box<TExpr>),
    TRecordConstructor(Ty, Vec<(Identifier, Box<TExpr>)>),
    TUnionConstructor(Ty, Identifier, Vec<(Identifier, Box<TExpr>)>),
    TSlotAccessor(Box<TExpr>, Identifier, Ty),
    TPointerSlotAccessor(Box<TExpr>, Identifier, Ty),
    TArrayIndex(Box<TExpr>, Box<TExpr>, Ty),
    TSpanIndex(Box<TExpr>, Box<TExpr>, Ty),
    TEmbed(Ty, String, Vec<Box<TExpr>>),
    TDeref(Box<TExpr>),
    TSizeOf(Ty),
}

pub struct TypedBinding {
    pub name: Identifier,
    pub ty: Ty,
    pub rename: Identifier,
}

pub enum CaseRef {
    CasePlain,
    CaseRefValue,
}

pub struct TypedWhen(Identifier, Vec<TypedBinding>, Box<TStmt>);

pub enum BorrowStmtKind {
    Read,
    Write,
    Reborrow,
}

pub struct ModuleName(String);
