use error::MalExceptionError;
use failure::Fallible;
use printer::pr_str;
use reader::read_str;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fs::File;
use std::io::stdout;
use std::io::Write;
use std::io::{stdin, Read};
use std::rc::Rc;
use time;
use types::{Closure, MalType, InnerMalType, ClosureEnv};

fn add(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "add should have 2 params");
    Ok(new_mal!(Num(
        params.pop_front().unwrap().to_number() + params.pop_front().unwrap().to_number()
    )))
}

fn minus(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "minus should have 2 params");
    Ok(new_mal!(Num(
        params.pop_front().unwrap().to_number() - params.pop_front().unwrap().to_number()
    )))
}

fn multiply(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "multiply should have 2 params");
    Ok(new_mal!(Num(
        params.pop_front().unwrap().to_number() * params.pop_front().unwrap().to_number()
    )))
}

fn divide(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "divide should have 2 params");
    Ok(new_mal!(Num(
        params.pop_front().unwrap().to_number() / params.pop_front().unwrap().to_number()
    )))
}

fn prn(params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    println!("{}", pr_str2(params, None)?.to_string());
    stdout().flush()?;
    Ok(new_mal!(Nil))
}

fn pr_str2(params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    Ok(new_mal!(String(
        params
            .into_iter()
            .map(|p| pr_str(&p, true))
            .collect::<Vec<String>>()
            .join(" ")
    )))
}

pub fn str2(params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    Ok(new_mal!(String(
        params
            .into_iter()
            .map(|p| pr_str(&p, false))
            .collect::<Vec<String>>()
            .join("")
    )))
}

fn println2(params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    println!(
        "{}",
        params
            .into_iter()
            .map(|p| pr_str(&p, false))
            .collect::<Vec<String>>()
            .join(" ")
    );
    stdout().flush()?;
    Ok(new_mal!(Nil))
}

#[allow(unused_mut)]
fn list(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    Ok(new_mal!(List(params, new_mal!(Nil))))
}

fn is_list(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "list? should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_list())))
}

fn is_empty(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "empty? should have 1 params");
    Ok(new_mal!(Bool(
        params.pop_front().unwrap().is_empty_collection()
    )))
}

fn count(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "count should have 1 params");
    let param = params.pop_front().unwrap();
    if param.is_nil() {
        return Ok(new_mal!(Num(0f64)));
    }
    ensure!(param.is_collection(), "param should be list");
    Ok(new_mal!(Num(param.len() as f64)))
}

fn equal(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "= should have 2 params");
    let left = params.pop_front().unwrap();
    let right = params.pop_front().unwrap();
    Ok(new_mal!(Bool(eq(left, right))))
}

fn eq(left: MalType, right: MalType) -> bool {
    if left.is_collection() && right.is_collection() {
        let inner_left = left.to_items_ref();
        let inner_right = right.to_items_ref();
        if inner_left.len() != inner_right.len() {
            return false;
        }

        inner_left
            .into_iter()
            .zip(inner_right)
            .all(|(l, r)| eq(l.clone(), r.clone()))
    } else if left.is_hashmap() && right.is_hashmap() {
        let inner_left = left.to_hashmap_ref();
        let inner_right = right.to_hashmap_ref();
        if inner_left.len() != inner_right.len() {
            return false;
        }

        inner_right
            .into_iter()
            .zip(inner_right)
            .all(|((lk, lv), (rk, rv))| lk == rk && eq(lv.clone(), rv.clone()))
    } else {
        return left == right;
    }
}

fn less_than(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "< should have 2 params");
    let left = params.pop_front().unwrap();
    let right = params.pop_front().unwrap();
    Ok(new_mal!(Bool(left.to_number() < right.to_number())))
}

fn less_than_equal(
    mut params: LinkedList<MalType>,
    _c_env: Option<ClosureEnv>,
) -> Fallible<MalType> {
    ensure!(params.len() == 2, "<= should have 2 params");
    let left = params.pop_front().unwrap();
    let right = params.pop_front().unwrap();
    Ok(new_mal!(Bool(left.to_number() <= right.to_number())))
}

fn greater_than(
    mut params: LinkedList<MalType>,
    _c_env: Option<ClosureEnv>,
) -> Fallible<MalType> {
    ensure!(params.len() == 2, "> should have 2 params");
    let left = params.pop_front().unwrap();
    let right = params.pop_front().unwrap();
    Ok(new_mal!(Bool(left.to_number() > right.to_number())))
}

fn greater_than_equal(
    mut params: LinkedList<MalType>,
    _c_env: Option<ClosureEnv>,
) -> Fallible<MalType> {
    ensure!(params.len() == 2, ">= should have 2 params");
    let left = params.pop_front().unwrap();
    let right = params.pop_front().unwrap();
    Ok(new_mal!(Bool(left.to_number() >= right.to_number())))
}

fn read_string(
    mut params: LinkedList<MalType>,
    _c_env: Option<ClosureEnv>,
) -> Fallible<MalType> {
    ensure!(params.len() == 1, "read_string should have 1 params");
    let p = params.pop_front().unwrap();
    let s = p.to_string();
    read_str(&s)
}

fn slurp(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "slurp should have 1 params");
    let p = params.pop_front().unwrap();
    let file_name = p.to_string();
    let mut file = File::open(&file_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(new_mal!(String(content)))
}

fn atom(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "atom should have 1 params");
    Ok(new_mal!(Atom(RefCell::new(
        params.pop_front().unwrap(),
    ))))
}

fn is_atom(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_atom should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_atom())))
}

fn deref(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "deref should have 1 params");
    let p = params.pop_front().unwrap();
    ensure!(
        p.is_atom(),
        "deref should have 1 param which is of type atom"
    );
    Ok(p.to_atom())
}

fn reset(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "reset should have 2 params");
    let atom = params.pop_front().unwrap();
    let new_value = params.pop_front().unwrap();
    ensure!(atom.is_atom(), "reset's first param should be of type atom");
    if let InnerMalType::Atom(a) = &*atom {
        let _ = a.replace(new_value.clone());
        return Ok(new_value);
    }

    unreachable!()
}

fn cons(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "cons should have 2 params");
    let first = params.pop_front().unwrap();
    let list = params.pop_front().unwrap();
    ensure!(list.is_collection(), "cons' second param should be list");
    let mut l = list.to_items();
    l.push_front(first);
    Ok(new_mal!(List(l, new_mal!(Nil))))
}

fn concat(params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(
        params.iter().all(|el| el.is_collection()),
        "concat's all params should be list"
    );
    let mut l = LinkedList::new();
    for mal in params {
        l.extend(&mut mal.to_items_ref().iter().cloned())
    }

    Ok(new_mal!(List(l, new_mal!(Nil))))
}

fn nth(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "nth should have 2 params");
    let list = params.pop_front().unwrap();
    let index_mal = params.pop_front().unwrap();
    ensure!(index_mal.is_num(), "nth's first param should be num");
    let float_index = index_mal.to_number();
    ensure!(
        float_index.trunc() == float_index,
        "nth index should be int"
    );
    let index = float_index.trunc() as usize;
    ensure!(list.is_collection(), "nth's second param should be list");
    let l = list.to_items_ref();
    ensure!(l.len() > index, "nth no enough items in list");
    Ok(l.iter().nth(index).unwrap().clone())
}

fn first(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "first should have 1 params");
    let list = params.pop_front().unwrap();
    if list.is_nil() || list.is_empty_collection() {
        return Ok(new_mal!(Nil));
    }
    ensure!(list.is_collection(), "first's param should be list or nil");
    let l = list.to_items_ref();
    Ok(l.front().unwrap().clone())
}

fn rest(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "rest should have 1 params");
    let list = params.pop_front().unwrap();
    if list.is_nil() || list.is_empty_collection() {
        return Ok(new_mal!(List(LinkedList::new(), new_mal!(Nil))));
    }
    ensure!(list.is_collection(), "rest's param should be list or nil");
    let mut l = list.to_items();
    l.pop_front().unwrap();
    Ok(new_mal!(List(l, new_mal!(Nil))))
}

fn throw(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "throw should have 1 params");
    let e = params.pop_front().unwrap();
    Err(MalExceptionError(pr_str(&e, true)).into())
}

fn apply(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() >= 2, "apply should have at least 2 params");
    let func = params.pop_front().unwrap();
    ensure!(func.is_closure(), "apply's first param should be func");
    let list = params.pop_back().unwrap();
    ensure!(list.is_collection(), "apply's last param should be list");
    params.extend(list.to_items_ref().iter().cloned());
    func.to_closure().call(params)
}

fn map(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() >= 2, "map should have 2 params");
    let func = params.pop_front().unwrap();
    ensure!(func.is_closure(), "map's first param should be func");
    let list = params.pop_front().unwrap();
    ensure!(list.is_collection(), "map's last param should be list");
    let f = func.to_closure();
    Ok(new_mal!(List(
        list.to_items_ref()
            .into_iter()
            .map(|mal| f.call(linked_list![mal.clone()]))
            .collect::<Fallible<LinkedList<MalType>>>()?,
        new_mal!(Nil)
    )))
}

fn is_nil(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "nil? should have 1 params");
    Ok(new_mal!(Bool(*params.pop_front().unwrap() == InnerMalType::Nil)))
}

fn is_true(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "true? should have 1 params");
    Ok(new_mal!(Bool(
        *params.pop_front().unwrap() == InnerMalType::Bool(true)
    )))
}

fn is_false(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "false? should have 1 params");
    Ok(new_mal!(Bool(
        *params.pop_front().unwrap() == InnerMalType::Bool(false)
    )))
}

fn is_symbol(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "symbol? should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_symbol())))
}

fn is_number(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "number? should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_num())))
}

fn is_fn(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "fn? should have 1 params");
    let p = params.pop_front().unwrap();
    Ok(new_mal!(Bool(p.is_closure() && !p.is_macro_closure())))
}

fn is_macro(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "macro? should have 1 params");
    Ok(new_mal!(Bool(
        params.pop_front().unwrap().is_macro_closure()
    )))
}

fn symbol(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "symbol should have 1 param");
    let s = params.pop_front().unwrap().to_string();
    Ok(new_mal!(Symbol(s)))
}

fn keyword(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "keyword should have 1 param");
    let s = params.pop_front().unwrap().to_string();
    Ok(new_mal!(Keyword(format!(":{}", s))))
}

fn is_keyword(
    mut params: LinkedList<MalType>,
    _c_env: Option<ClosureEnv>,
) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_keyword should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_keyword())))
}

fn vector(params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    Ok(new_mal!(Vec(params, new_mal!(Nil))))
}

fn is_vector(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_vector should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_vec())))
}

fn hashmap(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(
        params.len() % 2 == 0,
        "hashmap should have even number of params"
    );
    let mut map = HashMap::new();
    while let Some(key) = params.pop_front() {
        let value = params.pop_front().expect("get value");
        map.insert(key.to_hash_key(), value);
    }
    Ok(new_mal!(Hashmap(map, new_mal!(Nil))))
}

fn is_map(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_map should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_hashmap())))
}

fn is_string(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "string? should have 1 params");
    Ok(new_mal!(Bool(params.pop_front().unwrap().is_string())))
}

fn assoc(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(
        params.len() > 0 && params.len() % 2 == 1,
        "assoc should have odd params"
    );
    let mut map = params.pop_front().unwrap().to_hashmap();
    while let Some(key) = params.pop_front() {
        let value = params.pop_front().expect("get value");
        map.insert(key.to_hash_key(), value);
    }
    Ok(new_mal!(Hashmap(map, new_mal!(Nil))))
}

fn dissoc(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    let mut map = params.pop_front().unwrap().to_hashmap();
    let keys = params;
    for k in keys {
        map.remove(&k.to_hash_key());
    }
    Ok(new_mal!(Hashmap(map, new_mal!(Nil))))
}

fn get(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    let el = params.pop_front().unwrap();
    if el.is_nil() {
        return Ok(new_mal!(Nil));
    }
    let mut map = el.to_hashmap();
    let key = params.pop_front().unwrap();

    Ok(map.remove(&key.to_hash_key()).unwrap_or(new_mal!(Nil)))
}

fn contains(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    let map = params.pop_front().unwrap().to_hashmap();
    let key = params.pop_front().unwrap();
    Ok(new_mal!(Bool(map.contains_key(&key.to_hash_key()))))
}

fn keys(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    let mut map = params.pop_front().unwrap().to_hashmap();
    Ok(new_mal!(List(
        map.drain().map(|(k, _)| k.into_mal_type()).collect(),
        new_mal!(Nil)
    )))
}

fn vals(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    let mut map = params.pop_front().unwrap().to_hashmap();
    Ok(new_mal!(List(
        map.drain().map(|(_, v)| v).collect(),
        new_mal!(Nil)
    )))
}

fn is_sequential(
    mut params: LinkedList<MalType>,
    _c_env: Option<ClosureEnv>,
) -> Fallible<MalType> {
    let l = params.pop_front().unwrap();
    Ok(new_mal!(Bool(l.is_collection())))
}

fn readline(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "readline should have 1 params");
    let prompt = params.pop_front().unwrap();
    print!("{}", prompt.to_string());
    stdout().flush()?;
    let mut buf = String::new();
    let _ = stdin().read_line(&mut buf)?;
    if buf == "" {
        return Ok(new_mal!(Nil));
    }
    Ok(new_mal!(String(buf.trim_right().to_string())))
}

fn meta(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "meta should have 1 params");
    let s = params.pop_front().unwrap();
    Ok(s.get_metadata())
}

fn with_meta(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "with_meta should have 2 params");
    let source = params.pop_front().unwrap();
    let source = Rc::try_unwrap(source).unwrap_or_else(|source|(*source).clone());
    let metadata = params.pop_front().unwrap();
    Ok(match source {
        InnerMalType::List(l, ..) => new_mal!(List(l, metadata)),
        InnerMalType::Vec(l, ..) => new_mal!(Vec(l, metadata)),
        InnerMalType::Hashmap(l, ..) => new_mal!(Hashmap(l, metadata)),
        InnerMalType::Closure(l, ..) => new_mal!(Closure(l, metadata)),
        _ => unreachable!(),
    })
}

fn time_ms(params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 0, "time_ms should have 0 params");
    let t = time::get_time();
    Ok(new_mal!(Num(
        t.sec as f64 * 1000.0 + (t.nsec / 1000 / 1000) as f64
    )))
}

fn conj(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    let collection = Rc::try_unwrap(params.pop_front().unwrap()).unwrap_or_else(|s| (*s).clone());
    Ok(match collection {
        InnerMalType::Vec(l, meta) => {
            let mut l = l.clone();
            l.extend(params);
            new_mal!(Vec(l, meta.clone()))
        }
        InnerMalType::List(mut l, meta) => {
            for i in params {
                l.push_front(i);
            }
            new_mal!(List(l, meta.clone()))
        }
        _ => unreachable!(),
    })
}

fn swap(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() >= 2, "swap! should have more than 2 params");
    let atom = params.pop_front().unwrap();
    let func = params.pop_front().unwrap();
    ensure!(atom.is_atom(), "swap!'s first param should be of type atom");
    ensure!(func.is_closure(), "swap!'s second param should be a func");

    let old_mal = atom.to_atom();
    params.push_front(old_mal);
    let new_mal = func.to_closure().call(params)?;
    Ok(atom.replace_atom(new_mal)?)
}

fn seq(mut params: LinkedList<MalType>, _c_env: Option<ClosureEnv>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "seq should have 1 params");
    let p = Rc::try_unwrap(params.pop_front().unwrap()).unwrap_or_else(|s| (*s).clone());
    Ok(match p {
        InnerMalType::List(l, m) => {
            if !l.is_empty() {
                new_mal!(List(l, m))
            } else {
                new_mal!(Nil)
            }
        }
        InnerMalType::Vec(l, m) => {
            if !l.is_empty() {
                new_mal!(List(l, m))
            } else {
                new_mal!(Nil)
            }
        }
        InnerMalType::Nil => new_mal!(Nil),
        InnerMalType::String(s) => {
            if !s.is_empty() {
                new_mal!(List(
                    s.chars().map(|c| new_mal!(String(c.to_string()))).collect(),
                    new_mal!(Nil)
                ))
            } else {
                new_mal!(Nil)
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
