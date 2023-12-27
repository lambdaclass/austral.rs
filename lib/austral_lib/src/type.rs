pub enum Universe {
    FreeUniverse,
    LinearUniverse,
    TypeUniverse,
    RegionUniverse,
}

pub enum Ty {
    Unit,
    Boolean,
    Integer(Signedness, IntegerWidth),
    SingleFloat,
    DoubleFloat,
    // TODO NamedType(QIdent, Vec<Ty>, Universe),
    // TODO RegionTy(Region),
    ReadRef(Box<Ty>, Box<Ty>),
    WriteRef(Box<Ty>, Box<Ty>),
    Span(Box<Ty>, Box<Ty>),
    SpanMut(Box<Ty>, Box<Ty>),
    // TODO TyVar(TypeVar),
    Address(Box<Ty>),
    Pointer(Box<Ty>),
    FnPtr(Vec<Ty>, Box<Ty>),
    // TODO MonoTy(MonoId),
}

pub enum Signedness {
    Unsigned,
    Signed,
}

pub enum IntegerWidth {
    Width8,
    Width16,
    Width32,
    Width64,
    WidthByteSize,
    WidthIndex,
}
