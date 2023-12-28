use super::{
    ConstantDecl, ConstantDef, DocString, FunctionDecl, FunctionDef, ImportStmt, InstanceDecl,
    InstanceDef, RecordDecl, TypeClassDecl, TypeClassDef, TypeDecl, UnionDecl,
};
use crate::lexer::Token;
use chumsky::prelude::*;
use serde::{Deserialize, Serialize};

pub type ModuleDecl = ModuleBase<ModuleDeclItem>;
pub type ModuleDef = ModuleBase<ModuleDefItem>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ModuleBase<TModuleItem> {
    pub doc_string: Option<DocString>,
    pub imports: Vec<ImportStmt>,
    pub contents: Vec<TModuleItem>,
}

impl ModuleBase<ModuleDeclItem> {
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

impl ModuleBase<ModuleDefItem> {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        group((
            DocString::parser().or_not(),
            ImportStmt::parser().repeated().collect(),
            ModuleDefItem::parser().repeated().collect(),
        ))
        .map(|(doc_string, imports, contents)| Self {
            doc_string,
            imports,
            contents,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ModuleDefItem {
    Constant(ConstantDef),
    Function(FunctionDef),
    Instance(InstanceDef),
    Record(RecordDecl),
    Type(TypeDecl),
    TypeClass(TypeClassDef),
    Union(UnionDecl),
}

impl ModuleDefItem {
    pub fn parser<'a>() -> impl Clone + Parser<'a, &'a [Token<'a>], Self> {
        choice((
            ConstantDef::parser().map(Self::Constant),
            FunctionDef::parser().map(Self::Function),
            InstanceDef::parser().map(Self::Instance),
            RecordDecl::parser().map(Self::Record),
            TypeDecl::parser().map(Self::Type),
            TypeClassDef::parser().map(Self::TypeClass),
            UnionDecl::parser().map(Self::Union),
        ))
    }
}
