use types::MalType;

pub fn pr_str(mal: &MalType, print_readably: bool) -> String {
    let mut s = String::new();

    match mal {
        MalType::Symbol(sym) => s.push_str(sym),
        MalType::Nil => s.push_str("nil"),
        MalType::Bool(b) => s.push_str(&format!("{}", b)),
        MalType::Keyword(k) => s.push_str(k),
        MalType::String(k) => {
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
        MalType::Num(num) => s.push_str(&format!("{}", num)),
        MalType::List(list) => {
            s.push_str("(");
            for t in list {
                s.push_str(&pr_str(t, print_readably));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str(")");
        }
        MalType::Vec(list) => {
            s.push_str("[");
            for t in list {
                s.push_str(&pr_str(t, print_readably));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str("]");
        }
        MalType::Hashmap(hashmap) => {
            s.push_str("{");
            for k in hashmap {
                s.push_str(&pr_str(k, print_readably));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str("}");
        }
        MalType::WithMeta(vector, hashmap) => {
            s.push_str("(with-meta ");
            s.push_str(&pr_str(vector, print_readably));
            s.push_str(" ");
            s.push_str(&pr_str(hashmap, print_readably));
            s.push_str(")");
        }
        MalType::Atom(atom) => {
            s.push_str("(atom ");
            s.push_str(&pr_str(&atom.borrow(), print_readably));
            s.push_str(")")
        }
        MalType::Func(..) => {
            s.push_str("#<function>");
        }
        MalType::Closure(..) => {
            s.push_str("#<function>");
        }
    }

    s
}
