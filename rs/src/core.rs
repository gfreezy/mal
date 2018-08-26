use std::collections::HashMap;
use types::{MalType, CoreFunc};
use failure::Fallible;
use printer::pr_str;
use reader::read_str;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;


fn add(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "add should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() + params.remove(0).get_number()))
}

fn minus(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "minus should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() - params.remove(0).get_number()))
}


fn multiply(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "multiply should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() * params.remove(0).get_number()))
}


fn divide(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "divide should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() / params.remove(0).get_number()))
}

fn prn(params: Vec<MalType>) -> Fallible<MalType> {
    println!("{}", pr_str2(params)?.get_string());
    Ok(MalType::Nil)
}

fn pr_str2(params: Vec<MalType>) -> Fallible<MalType> {
    Ok(MalType::String(params.into_iter().map(|p| pr_str(&p, true)).collect::<Vec<String>>().join(" ")))
}

pub fn str2(params: Vec<MalType>) -> Fallible<MalType> {
    Ok(MalType::String(params.into_iter().map(|p| pr_str(&p, false)).collect::<Vec<String>>().join("")))
}

fn println2(params: Vec<MalType>) -> Fallible<MalType> {
    println!("{}", params.into_iter().map(|p| pr_str(&p, false)).collect::<Vec<String>>().join(" "));
    Ok(MalType::Nil)
}

#[allow(unused_mut)]
fn list(mut params: Vec<MalType>) -> Fallible<MalType> {
    Ok(MalType::List(params))
}


fn is_list(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "list? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_list()))
}

fn is_empty(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "empty? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_empty_collection()))
}

fn count(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "count should have 1 params");
    let param = params.remove(0);
    if param.is_nil() {
        return Ok(MalType::Num(0f64));
    }
    ensure!(param.is_collection(), "param should be list");
    Ok(MalType::Num(param.get_items().len() as f64))
}

fn equal(params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "= should have 2 params");
    Ok(MalType::Bool(eq(params)))
}

fn eq(mut params: Vec<MalType>) -> bool {
    let left = params.remove(0);
    let right = params.remove(0);
    if left.is_collection() && right.is_collection() {
        let inner_left = left.get_items();
        let inner_right = right.get_items();
        if inner_left.len() != inner_right.len() {
            return false;
        }

        return inner_left.into_iter().zip(inner_right).all(|(l, r)| eq(vec![l, r]));
    } else {
        return left == right;
    }
}

fn less_than(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "< should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() < right.get_number()))
}

fn less_than_equal(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "<= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() <= right.get_number()))
}

fn greater_than(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "> should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() > right.get_number()))
}

fn greater_than_equal(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, ">= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() >= right.get_number()))
}

fn read_string(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "read_string should have 1 params");
    let p = params.remove(0);
    let s = p.get_string();
    read_str(&s)
}

fn slurp(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "slurp should have 1 params");
    let p = params.remove(0);
    let file_name = p.get_string();
    let mut file = File::open(&file_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(MalType::String(content))
}

fn atom(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "atom should have 1 params");
    Ok(MalType::Atom(Rc::new(RefCell::new(params.remove(0)))))
}

fn is_atom(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "is_atom should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_atom()))
}

fn deref(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "deref should have 1 params");
    let p = params.remove(0);
    ensure!(p.is_atom(), "deref should have 1 param which is of type atom");
    Ok(p.get_atom())
}

fn reset(mut params: Vec<MalType>) -> Fallible<MalType> {
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

fn cons(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "cons should have 2 params");
    let first = params.remove(0);
    let list = params.remove(0);
    ensure!(list.is_collection(), "cons' second param should be list");
    let mut l = list.get_items();
    l.insert(0, first);
    Ok(MalType::List(l))
}

fn concat(params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.iter().all(|el| el.is_collection()), "concat's all params should be list");
    Ok(MalType::List(params.into_iter().map(|mal| mal.get_items()).collect::<Vec<Vec<MalType>>>().concat()))
}

fn nth(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 2, "nth should have 2 params");
    let list = params.remove(0);
    let index_mal = params.remove(0);
    ensure!(index_mal.is_num(), "nth's first param should be num");
    let float_index =  index_mal.get_number();
    ensure!(float_index.trunc() == float_index, "nth index should be int");
    let index = float_index.trunc() as usize;
    ensure!(list.is_collection(), "nth's second param should be list");
    let mut l = list.get_items();
    ensure!(l.len() > index, "nth no enough items in list");
    return Ok(l.remove(index))
}

fn first(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "first should have 1 params");
    let list = params.remove(0);
    if list.is_nil() || list.is_empty_collection() {
        return Ok(MalType::Nil);
    }
    ensure!(list.is_collection(), "first's param should be list or nil");
    let mut l = list.get_items();
    return Ok(l.remove(0))
}

fn rest(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "rest should have 1 params");
    let list = params.remove(0);
    if list.is_nil() || list.is_empty_collection() {
        return Ok(MalType::List(Vec::new()));
    }
    ensure!(list.is_collection(), "rest's param should be list or nil");
    let mut l = list.get_items();
    l.remove(0);
    return Ok(MalType::List(l))
}

fn throw(mut params: Vec<MalType>) -> Fallible<MalType> {
    ensure!(params.len() == 1, "throw should have 1 params");
    let e = params.remove(0);
    bail!("{}", pr_str(&e, false))
}

pub struct Ns {
    pub map: HashMap<String, CoreFunc>
}

impl Ns {
    pub fn new() -> Self {
        let mut map: HashMap<String, CoreFunc> = HashMap::new();
        map.insert("+".to_string(), add);
        map.insert("-".to_string(), minus);
        map.insert("*".to_string(), multiply);
        map.insert("/".to_string(), divide);
        map.insert("prn".to_string(), prn);
        map.insert("str".to_string(), str2);
        map.insert("pr-str".to_string(), pr_str2);
        map.insert("println".to_string(), println2);
        map.insert("list".to_string(), list);
        map.insert("list?".to_string(), is_list);
        map.insert("empty?".to_string(), is_empty);
        map.insert("count".to_string(), count);
        map.insert("=".to_string(), equal);
        map.insert("<".to_string(), less_than);
        map.insert("<=".to_string(), less_than_equal);
        map.insert(">".to_string(), greater_than);
        map.insert(">=".to_string(), greater_than_equal);
        map.insert("read-string".to_string(), read_string);
        map.insert("slurp".to_string(), slurp);
        map.insert("atom".to_string(), atom);
        map.insert("atom?".to_string(), is_atom);
        map.insert("deref".to_string(), deref);
        map.insert("reset!".to_string(), reset);
        map.insert("cons".to_string(), cons);
        map.insert("concat".to_string(), concat);
        map.insert("nth".to_string(), nth);
        map.insert("first".to_string(), first);
        map.insert("rest".to_string(), rest);
        map.insert("throw".to_string(), throw);

        Ns {
            map
        }
    }
}
