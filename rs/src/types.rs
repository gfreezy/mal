use env::Env;
use failure::Fallible;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::rc::Rc;

pub type ClosureFunc = fn(LinkedList<MalType>, Option<ClosureEnv>) -> Fallible<MalType>;

#[macro_export]
macro_rules! linked_list {
    ($($arg:expr),*) => {{
        let mut v: LinkedList<MalType> = LinkedList::new();
        $(v.push_back($arg);)*
        v
    }};
    ($($arg:expr),*,) => {{
        let mut v: LinkedList<MalType> = LinkedList::new();
        $(v.push_back($arg);)*
        v
    }}
}

#[derive(Debug, Clone, PartialEq)]
pub enum InnerMalType {
    List(LinkedList<MalType>, MalType),
    Vec(LinkedList<MalType>, MalType),
    Hashmap(HashMap<HashKey, MalType>, MalType),
    Num(f64),
    Symbol(String),
    Keyword(String),
    String(String),
    Nil,
    Bool(bool),

    Atom(RefCell<MalType>),
    Closure(Closure, MalType),
}

pub type MalType = Rc<InnerMalType>;

#[macro_export]
macro_rules! new_mal {
    ($t:tt($($arg:expr),*)) => {{
        Rc::new(InnerMalType::$t($($arg,)*))
    }};
    (Nil) => {{
        Rc::new(InnerMalType::Nil)
    }};
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub func: ClosureFunc,
    pub c_env: Option<ClosureEnv>,
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
            c_env,
            is_macro: false,
        }
    }

    pub fn call(&self, params: LinkedList<MalType>) -> Fallible<MalType> {
        let f = &self.func;
        f(params, self.c_env.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    String(String),
    Keyword(String),
}

impl HashKey {
    pub fn to_mal_type(&self) -> MalType {
        match *self {
            HashKey::String(ref s) => new_mal!(String(s.to_owned())),
            HashKey::Keyword(ref s) => new_mal!(Keyword(s.to_owned())),
        }
    }
    pub fn into_mal_type(self) -> MalType {
        match self {
            HashKey::String(s) => new_mal!(String(s)),
            HashKey::Keyword(s) => new_mal!(Keyword(s)),
        }
    }
}

impl InnerMalType {
    pub fn to_hash_key(&self) -> HashKey {
        match self {
            InnerMalType::String(s) => HashKey::String(s.clone()),
            InnerMalType::Keyword(s) => HashKey::Keyword(s.clone()),
            _ => unreachable!(),
        }
    }

    pub fn to_closure(&self) -> Closure {
        match self {
            InnerMalType::Closure(f, _) => f.clone(),
            _ => unreachable!(),
        }
    }

    pub fn to_symbol(&self) -> String {
        match self {
            InnerMalType::Symbol(s) => s.clone(),
            _ => unreachable!(),
        }
    }

    pub fn to_symbol_ref(&self) -> &String {
        match self {
            InnerMalType::Symbol(s) => s,
            _ => unreachable!(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            InnerMalType::String(s) => s.clone(),
            _ => unreachable!(),
        }
    }

    pub fn to_items(&self) -> LinkedList<MalType> {
        match self {
            InnerMalType::List(l, ..) => l.clone(),
            InnerMalType::Vec(l, ..) => l.clone(),
            _ => unreachable!(),
        }
    }

    pub fn to_items_ref(&self) -> &LinkedList<MalType> {
        match self {
            InnerMalType::List(l, ..) => l,
            InnerMalType::Vec(l, ..) => l,
            _ => unreachable!(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            InnerMalType::List(l, ..) => l.len(),
            InnerMalType::Vec(l, ..) => l.len(),
            _ => unreachable!(),
        }
    }

    pub fn to_hashmap(&self) -> HashMap<HashKey, MalType> {
        match self {
            InnerMalType::Hashmap(l, ..) => l.clone(),
            _ => unreachable!(),
        }
    }

    pub fn to_hashmap_ref(&self) -> &HashMap<HashKey, MalType> {
        match self {
            InnerMalType::Hashmap(l, ..) => l,
            _ => unreachable!(),
        }
    }

    pub fn to_symbol_list(&self) -> Vec<String> {
        let l = match *self {
            InnerMalType::List(ref l, ..) => l,
            InnerMalType::Vec(ref l, ..) => l,
            _ => unreachable!(),
        };
        l.iter().map(|el| el.to_symbol().to_owned()).collect()
    }

    pub fn to_number(&self) -> f64 {
        match self {
            InnerMalType::Num(n) => *n,
            _ => unreachable!(),
        }
    }

    pub fn to_atom(&self) -> MalType {
        match self {
            InnerMalType::Atom(mal) => {
                mal.clone().into_inner()
            }
            _ => unreachable!(),
        }
    }

    pub fn is_atom(&self) -> bool {
        if let &InnerMalType::Atom(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_symbol(&self) -> bool {
        if let &InnerMalType::Symbol(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_keyword(&self) -> bool {
        if let &InnerMalType::Keyword(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_num(&self) -> bool {
        if let &InnerMalType::Num(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_nil(&self) -> bool {
        if let InnerMalType::Nil = self {
            return true;
        }
        return false;
    }

    pub fn is_closure(&self) -> bool {
        if let &InnerMalType::Closure(..) = self {
            return true;
        }
        return false;
    }

    pub fn is_collection(&self) -> bool {
        return self.is_vec() || self.is_list();
    }

    pub fn is_empty_collection(&self) -> bool {
        return self.is_empty_list() || self.is_empty_vec();
    }

    pub fn is_list(&self) -> bool {
        if let &InnerMalType::List(..) = self {
            return true;
        }
        return false;
    }

    pub fn is_string(&self) -> bool {
        if let &InnerMalType::String(..) = self {
            return true;
        }
        return false;
    }

    pub fn is_vec(&self) -> bool {
        if let &InnerMalType::Vec(..) = self {
            return true;
        }
        return false;
    }

    pub fn is_hashmap(&self) -> bool {
        if let &InnerMalType::Hashmap(..) = self {
            return true;
        }
        return false;
    }
    pub fn is_empty_list(&self) -> bool {
        if let InnerMalType::List(ref list, ..) = *self {
            return list.is_empty();
        }
        return false;
    }

    pub fn is_empty_vec(&self) -> bool {
        if let InnerMalType::Vec(ref list, ..) = *self {
            return list.is_empty();
        }
        return false;
    }

    pub fn is_empty_hashmap(&self) -> bool {
        if let InnerMalType::Hashmap(ref list, ..) = *self {
            return list.is_empty();
        }
        return false;
    }

    pub fn did_collection_have_leading_symbol(&self) -> bool {
        if !self.is_collection() || self.is_empty_collection() {
            return false;
        }

        match *self {
            InnerMalType::List(ref l, ..) | InnerMalType::Vec(ref l, ..) => l.front().unwrap().is_symbol(),
            _ => unreachable!(),
        }
    }

    pub fn did_collection_have_leading_closure(&self) -> bool {
        if !self.is_collection() || self.is_empty_collection() {
            return false;
        }

        match *self {
            InnerMalType::List(ref l, ..) | InnerMalType::Vec(ref l, ..) => l.front().unwrap().is_closure(),
            _ => unreachable!(),
        }
    }

    pub fn get_first_symbol(&self) -> Option<&MalType> {
        if !self.is_collection() || self.is_empty_collection() {
            return None;
        }

        match *self {
            InnerMalType::List(ref l, ..) | InnerMalType::Vec(ref l, ..) => {
                if l.front().unwrap().is_symbol() {
                    l.front()
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn set_is_macro(&mut self) {
        match *self {
            InnerMalType::Closure(ref mut c, ..) => c.is_macro = true,
            _ => unreachable!(),
        }
    }

    pub fn is_macro_closure(&self) -> bool {
        match self {
            InnerMalType::Closure(c, ..) => c.is_macro,
            _ => false,
        }
    }

    pub fn get_metadata(&self) -> MalType {
        let m = match self {
            InnerMalType::List(_, metadata) => metadata,
            InnerMalType::Vec(_, metadata) => metadata,
            InnerMalType::Hashmap(_, metadata) => metadata,
            InnerMalType::Closure(_, metadata) => metadata,
            _ => unreachable!(),
        };
        m.clone()
    }

    pub fn replace_atom(&self, new: MalType) -> Fallible<MalType> {
        match self {
            InnerMalType::Atom(cell) => {
                cell.replace(new.clone());
                Ok(new)
            }
            _ => unreachable!()
        }
    }
}

impl<'a> From<&'a String> for InnerMalType {
    fn from(token: &String) -> Self {
        let mut new_token = String::with_capacity(token.capacity());

        let mut chars = token.chars().skip(1).peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.peek() {
                    Some('\\') => {
                        new_token.push('\\');
                        let _ = chars.next();
                    }
                    Some('n') => {
                        new_token.push('\n');
                        let _ = chars.next();
                    }
                    Some('t') => {
                        new_token.push('\t');
                        let _ = chars.next();
                    }
                    Some('"') => {
                        new_token.push('"');
                        let _ = chars.next();
                    }
                    _ => {
                        new_token.push('\\');
                    }
                }
            } else {
                new_token.push(c);
            }
        }

        let _ = new_token.pop();

        InnerMalType::String(new_token)
    }
}
