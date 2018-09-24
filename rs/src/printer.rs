use types::MalType;
use types::InnerMalType;
use std::rc::Rc;

pub fn pr_str(mal: &MalType, print_readably: bool) -> String {
    let mut s = String::new();

    match (**mal).clone() {
        InnerMalType::Symbol(sym) => s.push_str(sym.as_str()),
        InnerMalType::Nil => s.push_str("nil"),
        InnerMalType::Bool(b) => s.push_str(&format!("{}", b)),
        InnerMalType::Keyword(k) => s.push_str(k.as_str()),
        InnerMalType::String(k) => {
            if !print_readably {
                s.push_str(&k)
            } else {
                s.push('"');
                let mut chars = k.chars();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            s.push_str("\\\\");
                        }
                        '\n' => {
                            s.push_str("\\n");
                        }
                        '\t' => {
                            s.push_str("\\t");
                        }
                        '"' => {
                            s.push_str("\\\"");
                        }
                        _ => {
                            s.push(c);
                        }
                    }
                }
                s.push('"');
            }
        }
        InnerMalType::Num(num) => s.push_str(&format!("{}", num)),
        InnerMalType::List(list, _) => {
            s.push_str("(");
            for t in list {
                s.push_str(&pr_str(&t, print_readably));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str(")");
        }
        InnerMalType::Vec(list, _) => {
            s.push_str("[");
            for t in list {
                s.push_str(&pr_str(&t, print_readably));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str("]");
        }
        InnerMalType::Hashmap(hashmap, _) => {
            s.push_str("{");
            for (k, v) in hashmap.into_iter() {
                s.push_str(&pr_str(&k.to_mal_type(), print_readably));
                s.push_str(" ");
                s.push_str(&pr_str(&v, print_readably));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str("}");
        }
        InnerMalType::Atom(atom) => {
            s.push_str("(atom ");
            s.push_str(&pr_str(&atom.borrow(), print_readably));
            s.push_str(")")
        }
        InnerMalType::Closure(..) => {
            s.push_str("#<function>");
        }
    }

    s
}
