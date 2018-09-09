use std::cell::RefCell;
use fnv::FnvHashMap;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;
use types::MalType;

#[derive(Clone, Debug, PartialEq)]
pub struct EnvStruct {
    data: RefCell<FnvHashMap<String, MalType>>,
    outer: Option<Rc<EnvStruct>>,
}

pub type Env = Rc<EnvStruct>;

pub fn env_new(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MalType>) -> Env {
    let env = Rc::new(EnvStruct {
            data: RefCell::new(FnvHashMap::default()),
            outer
    });

    for (k, v) in binds.into_iter().zip(exprs) {
        env_set(env.clone(),k, v);
    }

    env
}

pub fn env_set(env: Env, key: String, value: MalType) {
    env.data.borrow_mut().insert(key, value);
}

pub fn env_find(mut env: Env, key: &str) -> Option<Env> {
    loop {
        if env.data.borrow().contains_key(key) {
            return Some(env);
        }
        if let Some(e) = env.outer.clone() {
            env = e;
        } else {
            return None;
        }
    }
}

pub fn env_get(env: Env, key: &str) -> Option<MalType> {
    if let Some(env) = env_find(env, key) {
        return env.data.borrow().get(key).cloned();
    }
    None
}

pub fn env_root(mut env: Env) -> Env {
    while let Some(e) = env.outer.clone() {
        env = e;
    }
    env
}


impl Display for EnvStruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:#?}", self.data.borrow());
        Ok(())
    }
}
