use std::collections::HashMap;
use types::MalType;

#[derive(Clone, Debug, PartialEq)]
pub struct Env {
    data: HashMap<String, MalType>,
    outer: Option<Box<Env>>
    // todo: child should inherit parent env, and read parent changes
}


impl Env {
    pub fn new(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MalType>) -> Self {
        let mut env = Env{
            data: HashMap::new(),
            outer: outer.map(|env| Box::new(env))
        };
        for (k, v) in binds.into_iter().zip(exprs) {
            env.set(k, v);
        }
        env
    }

    pub fn set(&mut self, key: String, value: MalType) -> MalType {
        self.data.insert(key, value.clone());
        value
    }

    pub fn find(&self, key: &str) -> Option<&Env> {
        if self.data.contains_key(key) {
            return Some(self)
        }

        return self.outer.as_ref().and_then(|env| env.find(key))
    }

    pub fn get(&self, key: &str) -> Option<&MalType> {
        let env = self.find(key);
        env.and_then(|env| env.data.get(key))
    }
}
