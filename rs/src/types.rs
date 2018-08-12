#[derive(Debug, Clone)]
pub enum MalType {
    List(Vec<MalType>),
    Vec(Vec<MalType>),
    Hashmap(Vec<MalType>),
    Num(f64),
    Symbol(String),
    Keyword(String),
    String(String),
    Quote(Box<MalType>),
    Quasiquote(Box<MalType>),
    Unquote(Box<MalType>),
    SpliceUnquote(Box<MalType>),
    WithMeta(Box<MalType>, Box<MalType>),
    Deref(String),
    Func(fn(f64, f64) -> f64),
}


impl MalType {
    pub fn get_func(self) -> fn(f64, f64) -> f64 {
        match self {
            MalType::Func(f) => f,
            _ => unreachable!()
        }
    }

    pub fn get_symbol(&self) -> &String {
        match self {
            MalType::Symbol(ref s) => s,
            _ => unreachable!()
        }
    }

    pub fn get_items(self) -> Vec<MalType> {
        match self {
            MalType::List(l) => l,
            MalType::Vec(l) => l,
            MalType::Hashmap(l) => l,
            _ => unreachable!()
        }
    }

    pub fn get_items_ref(&self) -> &Vec<MalType> {
        match *self {
            MalType::List(ref l) => l,
            MalType::Vec(ref l) => l,
            MalType::Hashmap(ref l) => l,
            _ => unreachable!()
        }
    }

    pub fn get_number(self) -> f64 {
        match self {
            MalType::Num(n) => n,
            _ => unreachable!()
        }
    }

    pub fn is_symbol(&self) -> bool {
        if let &MalType::Symbol(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_func(&self) -> bool {
        if let &MalType::Func(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_collection(&self) -> bool {
        return self.is_vec() || self.is_list() || self.is_hashmap();
    }

    pub fn is_empty_collection(&self) -> bool {
        return self.is_empty_list() || self.is_empty_vec() || self.is_empty_hashmap();
    }

    pub fn is_list(&self) -> bool {
        if let &MalType::List(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_vec(&self) -> bool {
        if let &MalType::Vec(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_hashmap(&self) -> bool {
        if let &MalType::Hashmap(_) = self {
            return true;
        }
        return false;
    }
    pub fn is_empty_list(&self) -> bool {
        if let &MalType::List(ref list) = self {
            return list.is_empty();
        }
        return false;
    }

    pub fn is_empty_vec(&self) -> bool {
        if let &MalType::Vec(ref list) = self {
            return list.is_empty();
        }
        return false;
    }

    pub fn is_empty_hashmap(&self) -> bool {
        if let &MalType::Hashmap(ref list) = self {
            return list.is_empty();
        }
        return false;
    }

    pub fn did_collection_have_leading_func(&self) -> bool {
        if self.is_empty_collection() {
            return false;
        }

        match *self {
            MalType::List(ref l) => l[0].is_func(),
            MalType::Vec(ref l) => l[0].is_func(),
            MalType::Hashmap(ref l) => l[0].is_func(),
            _ => unreachable!()
        }
    }

    pub fn get_first_symbol(&self) -> Option<&MalType> {
        if self.is_empty_collection() {
            return None;
        }

        match *self {
            MalType::List(ref l) | MalType::Vec(ref l) | MalType::Hashmap(ref l) => {
                if l[0].is_symbol() {
                    Some(&l[0])
                } else {
                    None
                }
            }
            _ => unreachable!()
        }
    }
}
