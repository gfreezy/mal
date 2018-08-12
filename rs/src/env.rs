use std::collections::HashMap;
use types::MalType;

#[derive(Clone, Debug)]
pub struct Env {
    data: HashMap<String, MalType>,
    outer: Option<Box<Env>>
}


impl Env {
    pub fn new(outer: Option<Env>) -> Self {
        return Env{
            data: HashMap::new(),
            outer: outer.map(|env| Box::new(env))
        }
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
