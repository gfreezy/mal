pub enum MalType {
    List(Vec<MalType>),
    Vec(Vec<MalType>),
    Hashmap(Vec<(MalType)>),
    Num(f64),
    Symbol(String),
    Keyword(String),
    Quote(Box<MalType>),
    Quasiquote(Box<MalType>),
    Unquote(Box<MalType>),
    SpliceUnquote(Box<MalType>),
    WithMeta(Box<MalType>, Box<MalType>),
    Deref(String),
}
