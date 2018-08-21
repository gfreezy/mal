use failure::Error;
use regex::Regex;
use types::MalType;

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

pub fn read_str(s: &str) -> Result<MalType, Error> {
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
    //    println!("{:?}", caps);
    caps
}

fn read_form(reader: &mut Reader) -> Result<MalType, Error> {
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
            Some(_) => return read_atom(reader),
        }
    } else {
        bail!("no token available")
    }
}

fn read_list(reader: &mut Reader) -> Result<MalType, Error> {
    let mut ret = Vec::new();
    loop {
        reader.next();

        let c = match reader.peek() {
            None => bail!("expected ')'"),
            Some(t) => t,
        };
        if c == ")" {
            return Ok(MalType::List(ret));
        }
        let type_ = read_form(reader)?;
        ret.push(type_);
    }
}

fn read_vec(reader: &mut Reader) -> Result<MalType, Error> {
    let mut ret = Vec::new();
    loop {
        reader.next();

        let c = match reader.peek() {
            None => bail!("expected ']'"),
            Some(t) => t,
        };
        if c == "]" {
            return Ok(MalType::Vec(ret));
        }
        let type_ = read_form(reader)?;
        ret.push(type_);
    }
}

fn read_hashmap(reader: &mut Reader) -> Result<MalType, Error> {
    let mut ret = Vec::new();
    loop {
        reader.next();

        let c = match reader.peek() {
            None => bail!("expected '}'"),
            Some(t) => t,
        };
        if c == "}" {
            return Ok(MalType::Hashmap(ret));
        }
        let type_ = read_form(reader)?;
        ret.push(type_);
    }
}

fn read_quote(reader: &mut Reader) -> Result<MalType, Error> {
    reader.next();
    Ok(MalType::Quote(Box::new(read_form(reader)?)))
}

fn read_quasiquote(reader: &mut Reader) -> Result<MalType, Error> {
    reader.next();
    Ok(MalType::Quasiquote(Box::new(read_form(reader)?)))
}

fn read_unquote(reader: &mut Reader) -> Result<MalType, Error> {
    reader.next();
    Ok(MalType::Unquote(Box::new(read_form(reader)?)))
}

fn read_splice_unquote(reader: &mut Reader) -> Result<MalType, Error> {
    reader.next();
    Ok(MalType::SpliceUnquote(Box::new(read_form(reader)?)))
}

fn read_atom(reader: &mut Reader) -> Result<MalType, Error> {
    match reader.peek() {
        None => unreachable!(),
        Some(token) => {
            if let Ok(num) = token.parse::<f64>() {
                return Ok(MalType::Num(num));
            }

            Ok(match token.as_ref() {
                "nil" => MalType::Nil,
                "true" => MalType::Bool(true),
                "false" => MalType::Bool(false),
                _ => MalType::Symbol(token.to_owned())
            })

        }
    }
}

fn read_string(reader: &mut Reader) -> Result<MalType, Error> {
    match reader.peek() {
        None => unreachable!(),
        Some(token) => {
            let mut new_token = String::with_capacity(token.capacity());

            let mut chars = token.chars().skip(1).peekable();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.peek() {
                        Some('\\') => {
                            new_token.push('\\');
                            let _ = chars.next();
                        },
                        Some('n') => {
                            new_token.push('\n');
                            let _ = chars.next();
                        },
                        Some('t') => {
                            new_token.push('\t');
                            let _ = chars.next();
                        },
                        Some('"') => {
                            new_token.push('"');
                            let _ = chars.next();
                        },
                        _ => {
                            new_token.push('\\');
                        }
                    }
                } else {
                    new_token.push(c);
                }
            }

            let _ = new_token.pop();

//            println!("{}", &new_token);
            return Ok(MalType::String(new_token));
        }
    }
}

fn read_keyword(reader: &mut Reader) -> Result<MalType, Error> {
    match reader.peek() {
        None => unreachable!(),
        Some(token) => {
            return Ok(MalType::Keyword(token.to_owned()));
        }
    }
}

fn read_with_meta(reader: &mut Reader) -> Result<MalType, Error> {
    reader.next();
    let hashmap = read_form(reader)?;
    reader.next();
    let vector = read_form(reader)?;
    return Ok(MalType::WithMeta(Box::new(vector), Box::new(hashmap)));
}

fn read_deref(reader: &mut Reader) -> Result<MalType, Error> {
    reader.next();
    match reader.peek() {
        None => unreachable!(),
        Some(token) => {
            return Ok(MalType::Deref(token.to_owned()));
        }
    }
}
