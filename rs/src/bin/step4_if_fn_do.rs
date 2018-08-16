#![feature(nll)]
#[macro_use]
extern crate failure;
extern crate rs;
extern crate rustyline;

use failure::Error;
use rs::env::Env;
use rs::printer::pr_str;
use rs::reader::read_str;
use rs::types::MalType;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rs::types::Closure;
use rs::core::Ns;

const HIST_PATH: &str = ".mal-history";


fn read(line: &str) -> Result<MalType, Error> {
    read_str(line)
}

fn eval(mal: MalType, env: &mut Env) -> Result<MalType, Error> {
    if !mal.is_list() || mal.is_empty_list() {
        return eval_ast(mal, env);
    }

    let mut list = mal.get_items();
    let first_mal = list.remove(0);

    if first_mal.is_symbol() {
        match first_mal.get_symbol_ref().as_ref() {
            "def!" => {
                ensure!(list.len() == 2, "def! should have 2 params");
                let symbol_key = list.remove(0).get_symbol();
                let value = eval(list.remove(0), env)?;
                return Ok(env.set(symbol_key, value))
            }
            "let*" => {
                ensure!(list.len() == 2, "let* should have 2 params");
                let mut new_env = Env::new(Some(env.clone()), Vec::new(), Vec::new());
                let mut binding_list = list.remove(0).get_items();
                ensure!(binding_list.len() % 2 == 0, "def! binding list should have 2n params");
                while binding_list.len() >= 2 {
                    let key = binding_list.remove(0).get_symbol();
                    let value = eval(binding_list.remove(0), &mut new_env)?;
                    new_env.set(key, value);
                }
                let formula = list.remove(0);
                return eval(formula, &mut new_env)
            }
            "do" => {
                let mut ret = MalType::Nil;
                for item in list {
                    ret = eval(item, env)?;
                }
                return Ok(ret)
            }
            "if" => {
                ensure!(list.len() >= 2, "if should have at least 2 params");
                ensure!(list.len() <= 3, "if should have at most 3 params");
                let condition_expr = list.remove(0);
                let then_clause = list.remove(0);
                let condition = eval(condition_expr, env)?;
                return match condition {
                    MalType::Nil | MalType::Bool(false) => {
                        if list.len() > 0 {
                            let else_clause = list.remove(0);
                            eval(else_clause, env)
                        } else {
                            Ok(MalType::Nil)
                        }
                    }
                    _ => eval(then_clause, env),
                }
            }
            "fn*" => {
                ensure!(list.len() == 2, "fn* should have 2 params");
                return Ok(MalType::Closure(Box::new(Closure {
                    env: env.clone(),
                    parameters: list.remove(0),
                    body: list.remove(0),
                })))
            },
            "env" => {
                println!("{:#?}", env);
                return Ok(MalType::Nil);
            }
            _ => {}
        };
    };

    let new_first_mal = eval(first_mal, env)?;
    match new_first_mal {
        MalType::Func(func) => {
            let params = list.into_iter().map(|el| eval(el, env)).collect::<Result<Vec<MalType>, Error>>()?;
            func(params)
        }
        MalType::Closure(closure) => {
            let params = closure.parameters.get_items();
            ensure!(list.len() == params.len() , "closure arguments not match params");
            let binds: Vec<String> = params.into_iter().map(|mal| mal.get_symbol()).collect();
            let exprs = list.into_iter().map(|el| eval(el, env)).collect::<Result<Vec<MalType>, Error>>()?;
            let mut new_env = Env::new(Some(closure.env), binds, exprs);
            eval(closure.body, &mut new_env)
        }
        _ => {
            let mut remind: Vec<MalType> = list.into_iter().map(|el| eval(el, env)).collect::<Result<Vec<MalType>, Error>>()?;
            remind.insert(0, new_first_mal);
            Ok(MalType::List(
                remind
            ))
        }
    }
}


fn print(s: &MalType) -> String {
    pr_str(s)
}

fn rep(s: &str, env: &mut Env) -> Result<String, Error> {
    Ok(print(&eval(read(s)?, env)?))
}

fn eval_ast(ast: MalType, env: &mut Env) -> Result<MalType, Error> {
    match ast {
        MalType::Symbol(s) => {
            return env.get(&s).map_or(Err(format_err!("'{}' not found", s)), |f| Ok(f.clone()));
        }
        MalType::List(list) => return Ok(MalType::List(list.into_iter().map(|el| eval(el, env)).collect::<Result<Vec<MalType>, Error>>()?)),
        MalType::Vec(list) => return Ok(MalType::Vec(list.into_iter().map(|el| eval(el, env)).collect::<Result<Vec<MalType>, Error>>()?)),
        MalType::Hashmap(list) => {
            let (keys, values): (Vec<(usize, MalType)>, Vec<(usize, MalType)>) =
                list.into_iter()
                    .enumerate()
                    .partition(|&(ref index, _)| index % 2 == 0);

            ensure!(keys.len() == values.len(), "not valid hashmap");
            let new_values: Vec<MalType> =
                values.into_iter()
                    .map(|(_, el)| eval(el, env))
                    .collect::<Result<Vec<MalType>, Error>>()?;

            let new_hashmap: Vec<MalType> =
                keys.into_iter()
                    .map(|(_, k)| k)
                    .zip(new_values)
                    .flat_map(|o| vec![o.0, o.1])
                    .collect();

            return Ok(MalType::Hashmap(new_hashmap));
        }
        _ => return Ok(ast)
    }
}

fn main() -> Result<(), Error> {
    let mut rl = Editor::<()>::new();
    if rl.load_history(HIST_PATH).is_err() {
        println!("No previous history.")
    }

    let ns = Ns::new();
    let mut repl_env = Env::new(None, Vec::new(), Vec::new());
    for (k, v) in ns.map {
        repl_env.set(k, MalType::Func(v));
    }

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                match rep(line.as_ref(), &mut repl_env) {
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
