use error::MalExceptionError;
use failure::Fallible;
use printer::pr_str;
use reader::read_str;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use types::ClosureEnv;
use types::{Closure, MalType};

fn add(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "add should have 2 params");
    Ok(MalType::Num(
        params.remove(0).get_number() + params.remove(0).get_number(),
    ))
}

fn minus(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "minus should have 2 params");
    Ok(MalType::Num(
        params.remove(0).get_number() - params.remove(0).get_number(),
    ))
}

fn multiply(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "multiply should have 2 params");
    Ok(MalType::Num(
        params.remove(0).get_number() * params.remove(0).get_number(),
    ))
}

fn divide(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "divide should have 2 params");
    Ok(MalType::Num(
        params.remove(0).get_number() / params.remove(0).get_number(),
    ))
}

fn prn(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    println!("{}", pr_str2(params, None)?.get_string());
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
    Ok(MalType::Nil)
}

#[allow(unused_mut)]
fn list(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    Ok(MalType::List(params))
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
    Ok(MalType::Num(param.get_items().len() as f64))
}

fn equal(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "= should have 2 params");
    Ok(MalType::Bool(eq(params, None)))
}

fn eq(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> bool {
    let left = params.remove(0);
    let right = params.remove(0);
    if left.is_collection() && right.is_collection() {
        let inner_left = left.get_items();
        let inner_right = right.get_items();
        if inner_left.len() != inner_right.len() {
            return false;
        }

        return inner_left
            .into_iter()
            .zip(inner_right)
            .all(|(l, r)| eq(vec![l, r], None));
    } else {
        return left == right;
    }
}

fn less_than(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "< should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() < right.get_number()))
}

fn less_than_equal(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "<= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() <= right.get_number()))
}

fn greater_than(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "> should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() > right.get_number()))
}

fn greater_than_equal(
    mut params: Vec<MalType>,
    _c_env: Option<Rc<ClosureEnv>>,
) -> Fallible<MalType> {
    ensure!(params.len() == 2, ">= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() >= right.get_number()))
}

fn read_string(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "read_string should have 1 params");
    let p = params.remove(0);
    let s = p.get_string();
    read_str(&s)
}

fn slurp(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "slurp should have 1 params");
    let p = params.remove(0);
    let file_name = p.get_string();
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
    Ok(p.get_atom())
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
    let mut l = list.get_items();
    l.insert(0, first);
    Ok(MalType::List(l))
}

fn concat(params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(
        params.iter().all(|el| el.is_collection()),
        "concat's all params should be list"
    );
    Ok(MalType::List(
        params
            .into_iter()
            .map(|mal| mal.get_items())
            .collect::<Vec<Vec<MalType>>>()
            .concat(),
    ))
}

fn nth(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "nth should have 2 params");
    let list = params.remove(0);
    let index_mal = params.remove(0);
    ensure!(index_mal.is_num(), "nth's first param should be num");
    let float_index = index_mal.get_number();
    ensure!(
        float_index.trunc() == float_index,
        "nth index should be int"
    );
    let index = float_index.trunc() as usize;
    ensure!(list.is_collection(), "nth's second param should be list");
    let mut l = list.get_items();
    ensure!(l.len() > index, "nth no enough items in list");
    return Ok(l.remove(index));
}

fn first(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "first should have 1 params");
    let list = params.remove(0);
    if list.is_nil() || list.is_empty_collection() {
        return Ok(MalType::Nil);
    }
    ensure!(list.is_collection(), "first's param should be list or nil");
    let mut l = list.get_items();
    return Ok(l.remove(0));
}

fn rest(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "rest should have 1 params");
    let list = params.remove(0);
    if list.is_nil() || list.is_empty_collection() {
        return Ok(MalType::List(Vec::new()));
    }
    ensure!(list.is_collection(), "rest's param should be list or nil");
    let mut l = list.get_items();
    l.remove(0);
    return Ok(MalType::List(l));
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
    params.extend(list.get_items());
    func.get_closure().call(params)
}

fn map(mut params: Vec<MalType>, _c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(params.len() >= 2, "map should have 2 params");
    let func = params.remove(0);
    ensure!(func.is_closure(), "map's first param should be func");
    let list = params.remove(0);
    ensure!(list.is_collection(), "map's last param should be list");
    let f = func.get_closure();
    Ok(MalType::List(
        list.get_items()
            .into_iter()
            .map(|mal| f.call(vec![mal]))
            .collect::<Fallible<Vec<MalType>>>()?,
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

        Ns { map: mapping }
    }
}
