use super::{
    ConstantDecl, DocString, FunctionDecl, ImportStmt, InstanceDecl, RecordDecl, TypeClassDecl,
    TypeDecl, UnionDecl,
};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Module {
    Decl(ModuleDecl),
    Def(ModuleDef),
}

impl Module {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            ModuleDecl::parser().map(Self::Decl),
            ModuleDef::parser().map(Self::Def),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ModuleDecl {
    doc_string: Option<DocString>,
    imports: Vec<ImportStmt>,
    contents: Vec<ModuleDeclItem>,
}

impl ModuleDecl {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            ImportStmt::parser().repeated().collect(),
            ModuleDeclItem::parser().repeated().collect(),
        ))
        .map(|(doc_string, imports, contents)| Self {
            doc_string,
            imports,
            contents,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ModuleDeclItem {
    Constant(ConstantDecl),
    Function(FunctionDecl),
    Instance(InstanceDecl),
    Record(RecordDecl),
    Type(TypeDecl),
    TypeClass(TypeClassDecl),
    Union(UnionDecl),
}

impl ModuleDeclItem {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            ConstantDecl::parser().map(Self::Constant),
            FunctionDecl::parser().map(Self::Function),
            InstanceDecl::parser().map(Self::Instance),
            RecordDecl::parser().map(Self::Record),
            TypeDecl::parser().map(Self::Type),
            TypeClassDecl::parser().map(Self::TypeClass),
            UnionDecl::parser().map(Self::Union),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ModuleDef {
    doc_string: Option<String>,
    imports: Vec<ImportStmt>,
    // contents: Vec<ModuleDefBody>,
}

impl ModuleDef {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        todo()
    }
}
