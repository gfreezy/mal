use generational_arena::{Arena, Index};
use std::cell::RefCell;
use fnv::FnvHashMap;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;
use types::MalType;

#[derive(Clone, Debug)]
struct Node {
    parent: Option<Index>,
    data: FnvHashMap<String, MalType>,
}


#[derive(Clone, Debug)]
pub struct Env {
    index: Index,
    arena: Rc<RefCell<Arena<Node>>>,
}


impl PartialEq for Env {
    fn eq(&self, other: &Env) -> bool {
        self.index == other.index
    }
}

impl Env {
    pub fn new(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MalType>) -> Self {
        let arena = match outer {
            None => Rc::new(RefCell::new(Arena::new())),
            Some(ref outer) => outer.arena.clone(),
        };
        let index = arena.borrow_mut().insert(
            Node {
                data: FnvHashMap::with_capacity_and_hasher(10, Default::default()),
                parent: outer.map(|e| e.index),
            }
        );
        let mut env = Env {
            index,
            arena,
        };
        for (k, v) in binds.into_iter().zip(exprs) {
            env.set(k, v);
        }

        env
    }

    pub fn set(&mut self, key: String, value: MalType) {
        let mut arena = self.arena.borrow_mut();
        let data = &mut arena[self.index].data;

        data.insert(key, value);
    }

    pub fn find(&self, key: &str) -> Option<Env> {
        let arena = self.arena.borrow();

        let mut index = self.index;
        loop {
            let node = arena.get(index);
            if let Some(true) = node.map(|map| map.data.contains_key(key)) {
                return Some(Env {
                    index,
                    arena: self.arena.clone(),
                });
            }
            index = match node.and_then(|n| n.parent) {
                Some(i) => i,
                None => return None
            };
        }
    }

    pub fn get(&self, key: &str) -> Option<MalType> {
        let env = self.find(key);

        env.and_then(|env| {
            let arena = env.arena.borrow();
            let node = arena.get(env.index).expect("get node");
            node.data.get(key).cloned()
        })
    }

    pub fn root(&self) -> Env {
        let arena = self.arena.borrow();

        let mut index = self.index;
        loop {
            let node = arena.get(index);
            match node.and_then(|n| n.parent) {
                Some(i) => index = i,
                None => return Env {
                    index,
                    arena: self.arena.clone(),
                }
            };
        }
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let arena = self.arena.borrow();
        let map = &arena.get(self.index).unwrap().data;
        write!(f, "{:#?}", map);
        Ok(())
    }
}
