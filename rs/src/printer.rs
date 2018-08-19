use types::MalType;

pub fn pr_str(mal: &MalType) -> String {
    let mut s = String::new();

    match mal {
        MalType::Symbol(sym) => s.push_str(sym),
        MalType::Nil => s.push_str("nil"),
        MalType::Bool(b) => s.push_str(&format!("{}", b)),
        MalType::Keyword(k) => s.push_str(k),
        MalType::String(k) => s.push_str(k),
        MalType::Deref(k) => s.push_str(&format!("(deref {})", k)),
        MalType::Num(num) => s.push_str(&format!("{}", num)),
        MalType::List(list) => {
            s.push_str("(");
            for t in list {
                s.push_str(&pr_str(t));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str(")");
        }
        MalType::Vec(list) => {
            s.push_str("[");
            for t in list {
                s.push_str(&pr_str(t));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str("]");
        }
        MalType::Hashmap(hashmap) => {
            s.push_str("{");
            for k in hashmap {
                s.push_str(&pr_str(k));
                s.push_str(" ");
            }
            s = s.trim().to_string();
            s.push_str("}");
        }
        MalType::Quote(l) => {
            s.push_str("(quote ");
            s.push_str(&pr_str(l));
            s.push_str(")")
        }
        MalType::Quasiquote(l) => {
            s.push_str("(quasiquote ");
            s.push_str(&pr_str(l));
            s.push_str(")")
        }
        MalType::Unquote(l) => {
            s.push_str("(unquote ");
            s.push_str(&pr_str(l));
            s.push_str(")")
        }
        MalType::SpliceUnquote(l) => {
            s.push_str("(splice-unquote ");
            s.push_str(&pr_str(l));
            s.push_str(")")
        }
        MalType::WithMeta(vector, hashmap) => {
            s.push_str("(with-meta ");
            s.push_str(&pr_str(vector));
            s.push_str(" ");
            s.push_str(&pr_str(hashmap));
            s.push_str(")");
        },
        MalType::Func(..) => {
            s.push_str("#<function>");
        },
        MalType::Closure(..) => {
//            s.push_str(&format!("{:#?}", c));
            s.push_str("#<function>");
        }
    }

    s
}
