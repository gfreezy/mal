use std::collections::HashMap;
use types::{MalType, CoreFunc};
use failure::Error;
use printer::pr_str;


pub struct Ns {
    pub map: HashMap<String, CoreFunc>
}


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

fn prn(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 1, "prn should have 1 params");
    println!("{}", pr_str(&params.remove(0)));
    Ok(MalType::Nil)
}

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

fn equal(mut params: Vec<MalType>) -> Result<MalType, Error> {
    ensure!(params.len() == 2, "= should have 2 params");
    let left = params.remove(0);
    let right = params.remove(0);
    Ok(MalType::Bool(left == right))
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

impl Ns {
    pub fn new() -> Self {
        let mut map: HashMap<String, CoreFunc> = HashMap::new();
        map.insert("+".to_string(), add);
        map.insert("-".to_string(), minus);
        map.insert("*".to_string(), multiply);
        map.insert("/".to_string(), divide);
        map.insert("prn".to_string(), prn);
        map.insert("list".to_string(), list);
        map.insert("list?".to_string(), is_list);
        map.insert("empty?".to_string(), is_empty);
        map.insert("count".to_string(), count);
        map.insert("=".to_string(), equal);
        map.insert("<".to_string(), less_than);
        map.insert("<=".to_string(), less_than_equal);
        map.insert(">".to_string(), greater_than);
        map.insert(">=".to_string(), greater_than_equal);
        Ns {
            map
        }
    }
}
