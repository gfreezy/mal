use error::CommentFoundError;
use failure::Fallible;
use regex::Regex;
use std::collections::HashMap;
use std::collections::LinkedList;
use types::{MalType, InnerMalType};
use std::rc::Rc;

struct Reader {
    tokens: Vec<String>,
    current_pos: usize,
}

impl Reader {
    fn new(tokens: Vec<String>) -> Self {
        Reader {
            tokens,
            current_pos: 0,
        }
    }

    fn next(&mut self) -> Option<&String> {
        let current = self.tokens.get(self.current_pos);
        self.current_pos += 1;
        current
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.get(self.current_pos)
    }
}

pub fn read_str(s: &str) -> Fallible<MalType> {
    let tokens = tokenizer(s);
    let mut reader = Reader::new(tokens);
    read_form(&mut reader)
}

fn tokenizer(s: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"#)
                .expect("make regexp");
    }

    let mut caps = Vec::new();
    for cap in RE.captures_iter(s) {
        caps.push(cap[1].to_string());
    }
    //        println!("{:?}", caps);
    caps
}

fn read_form(reader: &mut Reader) -> Fallible<MalType> {
    if let Some(token) = reader.peek() {
        let mut chars = token.chars();
        let first_char = chars.next();
        match first_char {
            None => bail!("token has no first char"),
            Some('(') => return read_list(reader),
            Some('[') => return read_vec(reader),
            Some('{') => return read_hashmap(reader),
            Some('\'') => return read_quote(reader),
            Some('`') => return read_quasiquote(reader),
            Some('~') => {
                let second_char = chars.next();
                match second_char {
                    Some('@') => return read_splice_unquote(reader),
                    Some(_) | None => return read_unquote(reader),
                }
            }
            Some(':') => return read_keyword(reader),
            Some('"') => return read_string(reader),
            Some('^') => return read_with_meta(reader),
            Some('@') => return read_deref(reader),
            Some(';') => return Err(CommentFoundError.into()),
            Some(_) => return read_symbol(reader),
        }
    } else {
        bail!("no token available")
    }
}

fn read_list(reader: &mut Reader) -> Fallible<MalType> {
    let mut ret = LinkedList::new();
    loop {
        reader.next();

        let c = match reader.peek() {
            None => bail!("expected ')'"),
            Some(t) => t,
        };
        if c == ")" {
            return Ok(new_mal!(List(ret, new_mal!(Nil))));
        }
        let type_ = match read_form(reader) {
            Ok(t) => t,
            Err(e) => {
                let _ = e.downcast::<CommentFoundError>()?;
                continue;
            }
        };
        ret.push_back(type_);
    }
}

fn read_vec(reader: &mut Reader) -> Fallible<MalType> {
    let mut ret = LinkedList::new();
    loop {
        reader.next();

        let c = match reader.peek() {
            None => bail!("expected ']'"),
            Some(t) => t,
        };
        if c == "]" {
            return Ok(new_mal!(Vec(ret, new_mal!(Nil))));
        }
        let type_ = match read_form(reader) {
            Ok(t) => t,
            Err(e) => {
                let _ = e.downcast::<CommentFoundError>()?;
                continue;
            }
        };
        ret.push_back(type_);
    }
}

fn read_hashmap(reader: &mut Reader) -> Fallible<MalType> {
    let mut ret: Vec<MalType> = Vec::new();
    loop {
        reader.next();

        let c = match reader.peek() {
            None => bail!("expected '}}'"),
            Some(t) => t,
        };
        if c == "}" {
            let mut mapping = HashMap::new();
            let mut drain = ret.drain(..);
            while let Some(key) = drain.next() {
                let value = drain.next().expect("get value");
                mapping.insert(key.to_hash_key(), value);
            }
            return Ok(new_mal!(Hashmap(mapping, new_mal!(Nil))));
        }
        let type_ = match read_form(reader) {
            Ok(t) => t,
            Err(e) => {
                let _ = e.downcast::<CommentFoundError>()?;
                continue;
            }
        };
        ret.push(type_);
    }
}

fn read_quote(reader: &mut Reader) -> Fallible<MalType> {
    reader.next();
    return Ok(new_mal!(List(
        linked_list![new_mal!(Symbol("quote".to_string())), read_form(reader)?],
        new_mal!(Nil)
    )));
}

fn read_quasiquote(reader: &mut Reader) -> Fallible<MalType> {
    reader.next();
    return Ok(new_mal!(List(
        linked_list![
            new_mal!(Symbol("quasiquote".to_string())),
            read_form(reader)?,
        ],
        new_mal!(Nil)
    )));
}

fn read_unquote(reader: &mut Reader) -> Fallible<MalType> {
    reader.next();
    return Ok(new_mal!(List(
        linked_list![new_mal!(Symbol("unquote".to_string())), read_form(reader)?],
        new_mal!(Nil)
    )));
}

fn read_splice_unquote(reader: &mut Reader) -> Fallible<MalType> {
    reader.next();
    return Ok(new_mal!(List(
        linked_list![
            new_mal!(Symbol("splice-unquote".to_string())),
            read_form(reader)?,
        ],
        new_mal!(Nil)
    )));
}

fn read_symbol(reader: &mut Reader) -> Fallible<MalType> {
    match reader.peek() {
        None => unreachable!(),
        Some(token) => {
            if let Ok(num) = token.parse::<f64>() {
                return Ok(new_mal!(Num(num)));
            }

            Ok(match token.as_ref() {
                "nil" => new_mal!(Nil),
                "true" => new_mal!(Bool(true)),
                "false" => new_mal!(Bool(false)),
                _ => new_mal!(Symbol(token.to_owned())),
            })
        }
    }
}

fn read_string(reader: &mut Reader) -> Fallible<MalType> {
    match reader.peek() {
        None => unreachable!(),
        Some(token) => Ok(new_mal!(from(token))),
    }
}

fn read_keyword(reader: &mut Reader) -> Fallible<MalType> {
    match reader.peek() {
        None => unreachable!(),
        Some(token) => {
            return Ok(new_mal!(Keyword(token.to_owned())));
        }
    }
}

fn read_with_meta(reader: &mut Reader) -> Fallible<MalType> {
    reader.next();
    let meta = read_form(reader)?;
    reader.next();
    let func = read_form(reader)?;
    return Ok(new_mal!(List(
        linked_list![new_mal!(Symbol("with-meta".to_string())), func, meta],
        new_mal!(Nil)
    )));
}

fn read_deref(reader: &mut Reader) -> Fallible<MalType> {
    reader.next();
    match reader.peek() {
        None => unreachable!(),
        Some(token) => {
            return Ok(new_mal!(List(
                linked_list![
                    new_mal!(Symbol("deref".to_string())),
                    new_mal!(Symbol(token.to_string())),
                ],
                new_mal!(Nil)
            )));
        }
    }
}
