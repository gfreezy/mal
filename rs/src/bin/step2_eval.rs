#[macro_use]
extern crate failure;
extern crate rs;
extern crate rustyline;

use failure::Fallible;
use rs::printer::pr_str;
use rs::reader::read_str;
use rs::types::MalType;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;

const HIST_PATH: &str = ".mal-history";

type ReplEnv = HashMap<String, fn(f64, f64) -> f64>;

fn add(a: f64, b: f64) -> f64 {
    a + b
}

fn minus(a: f64, b: f64) -> f64 {
    a - b
}

fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

fn divide(a: f64, b: f64) -> f64 {
    a / b
}

fn read(line: &str) -> Fallible<MalType> {
    read_str(line)
}

fn eval(s: MalType, env: &ReplEnv) -> Fallible<MalType> {
    let new_mal = eval_ast(s, env)?;
    if !new_mal.did_collection_have_leading_func() {
        return Ok(new_mal);
    }

    let new_list = new_mal.into_items();
    let func = new_list[0].clone().get_func();
    let args: Vec<f64> = new_list[1..]
        .iter()
        .map(|mal| mal.clone().into_number())
        .collect();

    Ok(MalType::Num(func(args[0], args[1])))
}

fn print(s: &MalType) -> String {
    pr_str(s, true)
}

fn rep(s: &str) -> Fallible<String> {
    let repl_env = {
        let mut map: ReplEnv = HashMap::new();
        map.insert("+".to_string(), add);
        map.insert("-".to_string(), minus);
        map.insert("*".to_string(), multiply);
        map.insert("/".to_string(), divide);
        map
    };
    Ok(print(&eval(read(s)?, &repl_env)?))
}

fn eval_ast(ast: MalType, env: &ReplEnv) -> Fallible<MalType> {
    match ast {
        MalType::Symbol(s) => {
            return env
                .get(&s)
                .map_or(Err(format_err!("'{}' not found", s)), |f| {
                    Ok(MalType::Func(f.clone()))
                })
        }
        MalType::List(list) => {
            return Ok(MalType::List(
                list.into_iter()
                    .map(|el| eval(el, env))
                    .collect::<Fallible<Vec<MalType>>>()?,
            ))
        }
        MalType::Vec(list) => {
            return Ok(MalType::Vec(
                list.into_iter()
                    .map(|el| eval(el, env))
                    .collect::<Fallible<Vec<MalType>>>()?,
            ))
        }
        MalType::Hashmap(list) => {
            let (keys, values): (Vec<(usize, MalType)>, Vec<(usize, MalType)>) = list
                .into_iter()
                .enumerate()
                .partition(|&(ref index, _)| index % 2 == 0);

            ensure!(keys.len() == values.len(), "not valid hashmap");
            let new_values: Vec<MalType> = values
                .into_iter()
                .map(|(_, el)| eval(el, env))
                .collect::<Fallible<Vec<MalType>>>()?;

            let new_hashmap: Vec<MalType> = keys
                .into_iter()
                .map(|(_, k)| k)
                .zip(new_values)
                .flat_map(|o| vec![o.0, o.1])
                .collect();

            return Ok(MalType::Hashmap(new_hashmap));
        }
        _ => return Ok(ast),
    }
}

fn main() -> Fallible<()> {
    let mut rl = Editor::<()>::new();
    if rl.load_history(HIST_PATH).is_err() {
        println!("No previous history.")
    }

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                match rep(line.as_ref()) {
                    Ok(s) => println!("{}", s),
                    Err(e) => println!("{}", e),
                }
            }
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    rl.save_history(HIST_PATH)?;
    Ok(())
}
