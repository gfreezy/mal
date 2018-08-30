use error::MalExceptionError;
use failure::Fallible;
use printer::pr_str;
use reader::read_str;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::stdout;
use std::io::Write;
use std::io::{stdin, Read};
use std::rc::Rc;
use time;
use types::ClosureEnv;
use types::{Closure, MalType};

fn add(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "add should have 2 params");
    Ok(MalType::Num(
        params.remove(0).into_number() + params.remove(0).into_number(),
    ))
}

fn minus(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "minus should have 2 params");
    Ok(MalType::Num(
        params.remove(0).into_number() - params.remove(0).into_number(),
    ))
}

fn multiply(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "multiply should have 2 params");
    Ok(MalType::Num(
        params.remove(0).into_number() * params.remove(0).into_number(),
    ))
}

fn divide(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "divide should have 2 params");
    Ok(MalType::Num(
        params.remove(0).into_number() / params.remove(0).into_number(),
    ))
}

fn prn(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    println!("{}", pr_str2(params, None)?.into_string());
    stdout().flush()?;
    Ok(MalType::Nil)
}

fn pr_str2(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    Ok(MalType::String(
        params
            .into_iter()
            .map(|p| pr_str(&p, true))
            .collect::<Vec<String>>()
            .join(" "),
    ))
}

pub fn str2(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    Ok(MalType::String(
        params
            .into_iter()
            .map(|p| pr_str(&p, false))
            .collect::<Vec<String>>()
            .join(""),
    ))
}

fn println2(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    println!(
        "{}",
        params
            .into_iter()
            .map(|p| pr_str(&p, false))
            .collect::<Vec<String>>()
            .join(" ")
    );
    stdout().flush()?;
    Ok(MalType::Nil)
}

#[allow(unused_mut)]
fn list(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    Ok(MalType::List(params, Box::new(MalType::Nil)))
}

fn is_list(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "list? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_list()))
}

fn is_empty(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "empty? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_empty_collection()))
}

fn count(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "count should have 1 params");
    let param = params.remove(0);
    if param.is_nil() {
        return Ok(MalType::Num(0f64));
    }
    ensure!(param.is_collection(), "param should be list");
    Ok(MalType::Num(param.into_items().len() as f64))
}

fn equal(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(eq(left, right)))
}

fn eq(left: MalType, right: MalType) -> bool {
    if left.is_collection() && right.is_collection() {
        let inner_left = left.into_items();
        let inner_right = right.into_items();
        if inner_left.len() != inner_right.len() {
            return false;
        }

        inner_left
            .into_iter()
            .zip(inner_right)
            .all(|(l, r)| eq(l, r))
    } else if left.is_hashmap() && right.is_hashmap() {
        let inner_left = left.into_hashmap();
        let mut inner_right = right.into_hashmap();
        if inner_left.len() != inner_right.len() {
            return false;
        }

        for (k, v) in inner_left {
            if let Some(rv) = inner_right.remove(&k) {
                if !eq(v, rv) {
                    return false;
                }
            }
        }

        return true;
    } else {
        return left == right;
    }
}

fn less_than(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "< should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.into_number() < right.into_number()))
}

fn less_than_equal(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "<= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.into_number() <= right.into_number()))
}

fn greater_than(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "> should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.into_number() > right.into_number()))
}

fn greater_than_equal(
    mut params: Vec<MalType>,
    _c_env: Option<Rc<ClosureEnv>>,
) -> Fallible<MalType> {
    ensure!(params.len() == 2, ">= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.into_number() >= right.into_number()))
}

fn read_string(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "read_string should have 1 params");
    let p = params.remove(0);
    let s = p.into_string();
    read_str(&s)
}

fn slurp(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "slurp should have 1 params");
    let p = params.remove(0);
    let file_name = p.into_string();
    let mut file = File::open(&file_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(MalType::String(content))
}

fn atom(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "atom should have 1 params");
    Ok(MalType::Atom(Rc::new(RefCell::new(params.remove(0)))))
}

fn is_atom(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_atom should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_atom()))
}

fn deref(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "deref should have 1 params");
    let p = params.remove(0);
    ensure!(
        p.is_atom(),
        "deref should have 1 param which is of type atom"
    );
    Ok(p.to_atom())
}

fn reset(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "reset should have 2 params");
    let atom = params.remove(0);
    let new_value = params.remove(0);
    ensure!(atom.is_atom(), "reset's first param should be of type atom");
    if let MalType::Atom(a) = atom {
        let _ = a.replace(new_value.clone());
        return Ok(new_value);
    }

    unreachable!()
}

fn cons(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "cons should have 2 params");
    let first = params.remove(0);
    let list = params.remove(0);
    ensure!(list.is_collection(), "cons' second param should be list");
    let mut l = list.into_items();
    l.insert(0, first);
    Ok(MalType::List(l, Box::new(MalType::Nil)))
}

fn concat(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(
        params.iter().all(|el| el.is_collection()),
        "concat's all params should be list"
    );
    Ok(MalType::List(
        params
            .into_iter()
            .map(|mal| mal.into_items())
            .collect::<Vec<Vec<MalType>>>()
            .concat(),
        Box::new(MalType::Nil),
    ))
}

fn nth(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "nth should have 2 params");
    let list = params.remove(0);
    let index_mal = params.remove(0);
    ensure!(index_mal.is_num(), "nth's first param should be num");
    let float_index = index_mal.into_number();
    ensure!(
        float_index.trunc() == float_index,
        "nth index should be int"
    );
    let index = float_index.trunc() as usize;
    ensure!(list.is_collection(), "nth's second param should be list");
    let mut l = list.into_items();
    ensure!(l.len() > index, "nth no enough items in list");
    Ok(l.remove(index))
}

fn first(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "first should have 1 params");
    let list = params.remove(0);
    if list.is_nil() || list.is_empty_collection() {
        return Ok(MalType::Nil);
    }
    ensure!(list.is_collection(), "first's param should be list or nil");
    let mut l = list.into_items();
    Ok(l.remove(0))
}

fn rest(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "rest should have 1 params");
    let list = params.remove(0);
    if list.is_nil() || list.is_empty_collection() {
        return Ok(MalType::List(Vec::new(), Box::new(MalType::Nil)));
    }
    ensure!(list.is_collection(), "rest's param should be list or nil");
    let mut l = list.into_items();
    l.remove(0);
    Ok(MalType::List(l, Box::new(MalType::Nil)))
}

fn throw(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "throw should have 1 params");
    let e = params.remove(0);
    Err(MalExceptionError(pr_str(&e, true)).into())
}

fn apply(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() >= 2, "apply should have at least 2 params");
    let func = params.remove(0);
    ensure!(func.is_closure(), "apply's first param should be func");
    let list = params.pop().unwrap();
    ensure!(list.is_collection(), "apply's last param should be list");
    params.extend(list.into_items());
    func.into_closure().call(params)
}

fn map(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() >= 2, "map should have 2 params");
    let func = params.remove(0);
    ensure!(func.is_closure(), "map's first param should be func");
    let list = params.remove(0);
    ensure!(list.is_collection(), "map's last param should be list");
    let f = func.into_closure();
    Ok(MalType::List(
        list.into_items()
            .into_iter()
            .map(|mal| f.call(vec![mal]))
            .collect::<Fallible<Vec<MalType>>>()?,
        Box::new(MalType::Nil),
    ))
}

fn is_nil(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "nil? should have 1 params");
    Ok(MalType::Bool(params.remove(0) == MalType::Nil))
}

fn is_true(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "true? should have 1 params");
    Ok(MalType::Bool(params.remove(0) == MalType::Bool(true)))
}

fn is_false(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "false? should have 1 params");
    Ok(MalType::Bool(params.remove(0) == MalType::Bool(false)))
}

fn is_symbol(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "symbol? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_symbol()))
}

fn is_number(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "number? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_num()))
}

fn is_fn(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "fn? should have 1 params");
    let p = params.remove(0);
    Ok(MalType::Bool(p.is_closure() && !p.is_macro_closure()))
}

fn is_macro(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "macro? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_macro_closure()))
}

fn symbol(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "symbol should have 1 param");
    let s = params.remove(0).into_string();
    Ok(MalType::Symbol(s))
}

fn keyword(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "keyword should have 1 param");
    let s = params.remove(0).into_string();
    Ok(MalType::Keyword(format!(":{}", s)))
}

fn is_keyword(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_keyword should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_keyword()))
}

fn vector(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    Ok(MalType::Vec(params, Box::new(MalType::Nil)))
}

fn is_vector(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_vector should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_vec()))
}

fn hashmap(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(
        params.len() % 2 == 0,
        "hashmap should have even number of params"
    );
    let mut map = HashMap::new();
    let mut drain = params.drain(..);
    while let Some(key) = drain.next() {
        let value = drain.next().expect("get value");
        map.insert(key.into_hash_key(), value);
    }
    Ok(MalType::Hashmap(map, Box::new(MalType::Nil)))
}

fn is_map(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_map should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_hashmap()))
}

fn is_string(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "string? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_string()))
}

fn assoc(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(
        params.len() > 0 && params.len() % 2 == 1,
        "assoc should have odd params"
    );
    let mut map = params.remove(0).into_hashmap();
    let mut pairs = params;
    let mut drain = pairs.drain(..);
    while let Some(key) = drain.next() {
        let value = drain.next().expect("get value");
        map.insert(key.into_hash_key(), value);
    }
    Ok(MalType::Hashmap(map, Box::new(MalType::Nil)))
}

fn dissoc(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    let mut map = params.remove(0).into_hashmap();
    let keys = params;
    for k in keys {
        map.remove(&k.into_hash_key());
    }
    Ok(MalType::Hashmap(map, Box::new(MalType::Nil)))
}

fn get(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    let el = params.remove(0);
    if el.is_nil() {
        return Ok(MalType::Nil);
    }
    let mut map = el.into_hashmap();
    let key = params.remove(0);

    Ok(map.remove(&key.into_hash_key()).unwrap_or(MalType::Nil))
}

fn contains(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    let map = params.remove(0).into_hashmap();
    let key = params.remove(0);
    Ok(MalType::Bool(map.contains_key(&key.into_hash_key())))
}

fn keys(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    let mut map = params.remove(0).into_hashmap();
    Ok(MalType::List(
        map.drain().map(|(k, _)| k.into_mal_type()).collect(),
        Box::new(MalType::Nil),
    ))
}

fn vals(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    let mut map = params.remove(0).into_hashmap();
    Ok(MalType::List(
        map.drain().map(|(_, v)| v).collect(),
        Box::new(MalType::Nil),
    ))
}

fn is_sequential(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    let l = params.remove(0);
    Ok(MalType::Bool(l.is_collection()))
}

fn readline(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "readline should have 1 params");
    let prompt = params.remove(0);
    print!("{}", prompt.into_string());
    stdout().flush()?;
    let mut buf = String::new();
    let _ = stdin().read_line(&mut buf)?;
    if buf == "" {
        return Ok(MalType::Nil);
    }
    Ok(MalType::String(buf.trim_right().to_string()))
}

fn meta(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "meta should have 1 params");
    let s = params.remove(0);
    Ok(s.get_metadata())
}

fn with_meta(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "with_meta should have 2 params");
    let source = params.remove(0);
    let metadata = params.remove(0);
    Ok(match source {
        MalType::List(l, ..) => MalType::List(l, Box::new(metadata)),
        MalType::Vec(l, ..) => MalType::Vec(l, Box::new(metadata)),
        MalType::Hashmap(l, ..) => MalType::Hashmap(l, Box::new(metadata)),
        MalType::Closure(l, ..) => MalType::Closure(l, Box::new(metadata)),
        _ => unreachable!(),
    })
}

fn time_ms(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 0, "time_ms should have 0 params");
    let t = time::get_time();
    Ok(MalType::Num(
        t.sec as f64 * 1000.0 + (t.nsec / 1000 / 1000) as f64,
    ))
}

fn conj(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    let collection = params.remove(0);
    Ok(match collection {
        MalType::Vec(mut l, meta) => {
            l.extend(params);
            MalType::Vec(l, meta)
        }
        MalType::List(mut l, meta) => {
            for i in params {
                l.insert(0, i);
            }
            MalType::List(l, meta)
        }
        _ => unreachable!(),
    })
}

fn swap(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() >= 2, "swap! should have more than 2 params");
    let atom = params.remove(0);
    let func = params.remove(0);
    ensure!(atom.is_atom(), "swap!'s first param should be of type atom");
    ensure!(func.is_closure(), "swap!'s second param should be a func");

    let old_mal = atom.to_atom();
    params.insert(0, old_mal);
    if let MalType::Atom(a) = atom {
        let new_mal = func.into_closure().call(params)?;
        let _ = a.replace(new_mal.clone());
        return Ok(new_mal);
    }

    unreachable!()
}

fn seq(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "seq should have 1 params");
    let p = params.remove(0);
    Ok(match p {
        MalType::List(l, m) => {
            if !l.is_empty() {
                MalType::List(l, m)
            } else {
                MalType::Nil
            }
        }
        MalType::Vec(l, m) => {
            if !l.is_empty() {
                MalType::List(l, m)
            } else {
                MalType::Nil
            }
        }
        MalType::Nil => MalType::Nil,
        MalType::String(s) => {
            if !s.is_empty() {
                MalType::List(
                    s.chars().map(|c| MalType::String(c.to_string())).collect(),
                    Box::new(MalType::Nil),
                )
            } else {
                MalType::Nil
            }
        }
        _ => unreachable!(),
    })
}
pub struct Ns {
    pub map: HashMap<String, Closure>,
}

impl Ns {
    pub fn new() -> Self {
        let mut mapping: HashMap<String, Closure> = HashMap::new();
        mapping.insert("+".to_string(), Closure::new(add, None));
        mapping.insert("-".to_string(), Closure::new(minus, None));
        mapping.insert("*".to_string(), Closure::new(multiply, None));
        mapping.insert("/".to_string(), Closure::new(divide, None));
        mapping.insert("prn".to_string(), Closure::new(prn, None));
        mapping.insert("str".to_string(), Closure::new(str2, None));
        mapping.insert("pr-str".to_string(), Closure::new(pr_str2, None));
        mapping.insert("println".to_string(), Closure::new(println2, None));
        mapping.insert("list".to_string(), Closure::new(list, None));
        mapping.insert("list?".to_string(), Closure::new(is_list, None));
        mapping.insert("empty?".to_string(), Closure::new(is_empty, None));
        mapping.insert("count".to_string(), Closure::new(count, None));
        mapping.insert("=".to_string(), Closure::new(equal, None));
        mapping.insert("<".to_string(), Closure::new(less_than, None));
        mapping.insert("<=".to_string(), Closure::new(less_than_equal, None));
        mapping.insert(">".to_string(), Closure::new(greater_than, None));
        mapping.insert(">=".to_string(), Closure::new(greater_than_equal, None));
        mapping.insert("read-string".to_string(), Closure::new(read_string, None));
        mapping.insert("slurp".to_string(), Closure::new(slurp, None));
        mapping.insert("atom".to_string(), Closure::new(atom, None));
        mapping.insert("atom?".to_string(), Closure::new(is_atom, None));
        mapping.insert("deref".to_string(), Closure::new(deref, None));
        mapping.insert("reset!".to_string(), Closure::new(reset, None));
        mapping.insert("cons".to_string(), Closure::new(cons, None));
        mapping.insert("concat".to_string(), Closure::new(concat, None));
        mapping.insert("nth".to_string(), Closure::new(nth, None));
        mapping.insert("first".to_string(), Closure::new(first, None));
        mapping.insert("rest".to_string(), Closure::new(rest, None));
        mapping.insert("throw".to_string(), Closure::new(throw, None));
        mapping.insert("map".to_string(), Closure::new(map, None));
        mapping.insert("apply".to_string(), Closure::new(apply, None));
        mapping.insert("nil?".to_string(), Closure::new(is_nil, None));
        mapping.insert("true?".to_string(), Closure::new(is_true, None));
        mapping.insert("false?".to_string(), Closure::new(is_false, None));
        mapping.insert("symbol?".to_string(), Closure::new(is_symbol, None));
        mapping.insert("symbol".to_string(), Closure::new(symbol, None));
        mapping.insert("keyword".to_string(), Closure::new(keyword, None));
        mapping.insert("keyword?".to_string(), Closure::new(is_keyword, None));
        mapping.insert("vector".to_string(), Closure::new(vector, None));
        mapping.insert("vector?".to_string(), Closure::new(is_vector, None));
        mapping.insert("hash-map".to_string(), Closure::new(hashmap, None));
        mapping.insert("map?".to_string(), Closure::new(is_map, None));
        mapping.insert("number?".to_string(), Closure::new(is_number, None));
        mapping.insert("string?".to_string(), Closure::new(is_string, None));
        mapping.insert("assoc".to_string(), Closure::new(assoc, None));
        mapping.insert("dissoc".to_string(), Closure::new(dissoc, None));
        mapping.insert("get".to_string(), Closure::new(get, None));
        mapping.insert("contains?".to_string(), Closure::new(contains, None));
        mapping.insert("keys".to_string(), Closure::new(keys, None));
        mapping.insert("vals".to_string(), Closure::new(vals, None));
        mapping.insert("sequential?".to_string(), Closure::new(is_sequential, None));
        mapping.insert("readline".to_string(), Closure::new(readline, None));
        mapping.insert("meta".to_string(), Closure::new(meta, None));
        mapping.insert("with-meta".to_string(), Closure::new(with_meta, None));
        mapping.insert("conj".to_string(), Closure::new(conj, None));
        mapping.insert("seq".to_string(), Closure::new(seq, None));
        mapping.insert("fn?".to_string(), Closure::new(is_fn, None));
        mapping.insert("macro?".to_string(), Closure::new(is_macro, None));
        mapping.insert("time-ms".to_string(), Closure::new(time_ms, None));
        mapping.insert("swap!".to_string(), Closure::new(swap, None));

        Ns { map: mapping }
    }
}
