use std::collections::HashMap;
use types::{MalType, CoreFunc};
use failure::Error;
use printer::pr_str;
use reader::read_str;
use std::fs::File;
use std::io::Read;


fn add(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "add should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() + params.remove(0).get_number()))
}

fn minus(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "minus should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() - params.remove(0).get_number()))
}


fn multiply(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "multiply should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() * params.remove(0).get_number()))
}


fn divide(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "divide should have 2 params");
    Ok(MalType::Num(params.remove(0).get_number() / params.remove(0).get_number()))
}

fn prn(params: Vec<MalType>) -> Result<MalType, Error> {
    println!("{}", pr_str2(params)?.get_string());
    Ok(MalType::Nil)
}

fn pr_str2(params: Vec<MalType>) -> Result<MalType, Error> {
    Ok(MalType::String(params.into_iter().map(|p| pr_str(&p, true)).collect::<Vec<String>>().join(" ")))
}

fn str2(params: Vec<MalType>) -> Result<MalType, Error> {
    Ok(MalType::String(params.into_iter().map(|p| pr_str(&p, false)).collect::<Vec<String>>().join("")))
}

fn println2(params: Vec<MalType>) -> Result<MalType, Error> {
    println!("{}", params.into_iter().map(|p| pr_str(&p, false)).collect::<Vec<String>>().join(" "));
    Ok(MalType::Nil)
}

#[allow(unused_mut)]
fn list(mut params: Vec<MalType>) -> Result<MalType, Error> {
    Ok(MalType::List(params))
}


fn is_list(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 1, "list? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_list()))
}

fn is_empty(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 1, "empty? should have 1 params");
    Ok(MalType::Bool(params.remove(0).is_empty_collection()))
}

fn count(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 1, "count should have 1 params");
    let param = params.remove(0);
    if param.is_nil() {
        return Ok(MalType::Num(0f64));
    }
    ensure!(param.is_collection(), "param should be list");
    Ok(MalType::Num(param.get_items().len() as f64))
}

fn equal(params: Vec<MalType>) -> Result<MalType, Error> {
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

fn less_than(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "< should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() < right.get_number()))
}

fn less_than_equal(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "<= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() <= right.get_number()))
}

fn greater_than(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "> should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() > right.get_number()))
}

fn greater_than_equal(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, ">= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left.get_number() >= right.get_number()))
}

fn read_string(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 1, "read_string should have 1 params");
    let p = params.remove(0);
    let s = p.get_string();
    read_str(&s)
}

fn slurp(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 1, "slurp should have 1 params");
    let p = params.remove(0);
    let file_name = p.get_string();
    let mut file = File::open(&file_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(MalType::String(content))
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

        Ns {
            map
        }
    }
}
