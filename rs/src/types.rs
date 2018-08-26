use env::Env;
use failure::Fallible;
use std::cell::RefCell;
use std::rc::Rc;


pub type ClosureFunc = fn(Vec<MalType>, Option<Rc<ClosureEnv>>) -> Fallible<MalType>;

#[derive(Debug, Clone, PartialEq)]
pub enum MalType {
    List(Vec<MalType>),
    Vec(Vec<MalType>),
    Hashmap(Vec<MalType>),
    Num(f64),
    Symbol(String),
    Keyword(String),
    String(String),
    WithMeta(Box<MalType>, Box<MalType>),
    Nil,
    Bool(bool),

    Atom(Rc<RefCell<MalType>>),
    Closure(Closure),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub func: ClosureFunc,
    pub c_env: Option<Rc<ClosureEnv>>,
    pub is_macro: bool,
}

#[derive(DebugStub, Clone, PartialEq)]
pub struct ClosureEnv {
    pub parameters: MalType,
    pub body: MalType,
    #[debug_stub = ".."]
    pub env: Env,
}

impl ClosureEnv {
    pub fn new(params: MalType, body: MalType, env: Env) -> Self {
        ClosureEnv {
            parameters: params,
            body,
            env,
        }
    }
}

impl Closure {
    pub fn new(func: ClosureFunc, c_env: Option<ClosureEnv>) -> Self {
        Closure {
            func,
            c_env: c_env.map(Rc::new),
            is_macro: false,
        }
    }
}


impl MalType {
    pub fn get_num(self) -> f64 {
        match self {
            MalType::Num(n) => n,
            _ => unreachable!()
        }
    }

    pub fn get_closure(self) -> Closure {
        match self {
            MalType::Closure(f) => f,
            _ => unreachable!()
        }
    }

    pub fn get_symbol(self) -> String {
        match self {
            MalType::Symbol(s) => s,
            _ => unreachable!()
        }
    }

    pub fn get_symbol_ref(&self) -> &String {
        match *self {
            MalType::Symbol(ref s) => s,
            _ => unreachable!()
        }
    }

    pub fn get_string(self) -> String {
        match self {
            MalType::String(s) => s,
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

    pub fn get_symbol_list_ref(&self) -> Vec<String> {
        let l = match *self {
            MalType::List(ref l) => l,
            MalType::Vec(ref l) => l,
            MalType::Hashmap(ref l) => l,
            _ => unreachable!()
        };
        l.iter().map(|el| el.get_symbol_ref().to_owned()).collect()
    }

    pub fn get_number(self) -> f64 {
        match self {
            MalType::Num(n) => n,
            _ => unreachable!()
        }
    }

    pub fn get_atom(&self) -> MalType {
        match *self {
            MalType::Atom(ref mal) => mal.borrow().clone(),
            _ => unreachable!()
        }
    }

    pub fn is_atom(&self) -> bool {
        if let &MalType::Atom(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_symbol(&self) -> bool {
        if let &MalType::Symbol(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_num(&self) -> bool {
        if let &MalType::Num(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_nil(&self) -> bool {
        if let MalType::Nil = self {
            return true;
        }
        return false;
    }

    pub fn is_closure(&self) -> bool {
        if let &MalType::Closure(_) = self {
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

    pub fn did_collection_have_leading_symbol(&self) -> bool {
        if !self.is_collection() || self.is_empty_collection() {
            return false;
        }

        match *self {
            MalType::List(ref l) | MalType::Vec(ref l) | MalType::Hashmap(ref l) => {
                l[0].is_symbol()
            }
            _ => unreachable!()
        }
    }

    pub fn did_collection_have_leading_closure(&self) -> bool {
        if !self.is_collection() || self.is_empty_collection() {
            return false;
        }

        match *self {
            MalType::List(ref l) | MalType::Vec(ref l) | MalType::Hashmap(ref l) => {
                l[0].is_closure()
            }
            _ => unreachable!()
        }
    }

    pub fn get_first_symbol(&self) -> Option<&MalType> {
        if !self.is_collection() || self.is_empty_collection() {
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

    pub fn set_is_macro(&mut self) {
        match *self {
            MalType::Closure(ref mut c) => {
                c.is_macro = true
            }
            _ => unreachable!()
        }
    }

    pub fn is_macro_closure(&self) -> bool {
        match *self {
            MalType::Closure(ref c) => {
                c.is_macro
            }
            _ => unreachable!()
        }
    }
}
