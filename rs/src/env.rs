use std::collections::HashMap;
use types::MalType;
use indextree::{Arena, NodeId};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;


#[derive(Clone, Debug, PartialEq)]
pub struct Env {
    node_id: NodeId,
    arena: Rc<RefCell<Arena<HashMap<String, MalType>>>>
}


impl Env {
    pub fn new(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MalType>) -> Self {
        let arena = match outer {
            None => Rc::new(RefCell::new(Arena::new())),
            Some(ref outer) => {
                outer.arena.clone()
            }
        };
        let node_id = arena.borrow_mut().new_node(HashMap::new());
        let mut env = Env {
            node_id,
            arena: arena.clone(),
        };
        for (k, v) in binds.into_iter().zip(exprs) {
            env.set(k, v);
        }

        if let Some(outer) = outer {
            outer.node_id.append(node_id, &mut arena.borrow_mut())
        }
        env
    }

    pub fn set(&mut self, key: String, value: MalType) -> MalType {
        let mut arena = self.arena.borrow_mut();
        let data = &mut arena.get_mut(self.node_id).expect("get data").data;

        data.insert(key, value.clone());
        value
    }

    pub fn find(&self, key: &str) -> Option<Env> {
        let arena = self.arena.borrow();
        let mut ancestors = self.node_id.ancestors(&arena);
        if let Some(node_id) = ancestors.find(|node_id| arena.get(*node_id).unwrap().data.contains_key(key)) {
            return Some(Env {
                node_id,
                arena: self.arena.clone()
            })
        }
        return None
    }

    pub fn get(&self, key: &str) -> Option<MalType> {
        let env = self.find(key);

        let ret = env.and_then(|env| {
            let arena = env.arena.borrow();
            let node = arena.get(env.node_id).expect("get node");
            node.data.get(key).map(|d| d.clone())
        });
        ret
    }

    pub fn root(&self) -> Env {
        let root = self.node_id.ancestors(&*self.arena.borrow()).last();
        Env {
            node_id: root.expect("no root node"),
            arena: self.arena.clone(),
        }
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let arena = self.arena.borrow();
         for node_id in self.node_id.ancestors(&arena) {
//             println!("{:?}", node_id);
             let map = &arena.get(node_id).unwrap().data;
             write!(f, "{:#?}", map);
         }
        Ok(())
    }
}
