#![feature(nll)]
extern crate core;
#[macro_use]
extern crate failure;
//#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate rs;
extern crate rustyline;

use failure::Fallible;
use rs::core::Ns;
use rs::env::Env;
use rs::printer::pr_str;
use rs::reader::read_str;
use rs::types::Closure;
use rs::types::MalType;
use rustyline::Editor;
use std::env;
use rs::error::CommentFoundError;
use rustyline::error::ReadlineError;
use rs::types::ClosureEnv;
use std::rc::Rc;


const HIST_PATH: &str = ".mal-history";

fn call(params: Vec<MalType>, c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(c_env.is_some(), "closure env should be available");
    let c_env = c_env.unwrap();
    let mut exprs = params;
    let mut binds = c_env.parameters.get_symbol_list_ref();

    let idx = binds.iter().position(|e| *e == "&");

    let new_env = if let Some(idx) = idx {
        ensure!(binds.len() == idx + 2, "& must be followed by a param name");
        ensure!(exprs.len() >= idx, "closure arguments not match params");

        // drop "&"
        binds.remove(idx);
        let positioned_args: Vec<MalType> = exprs.drain(0..idx).collect();
        let varargs = exprs;
        let mut exprs = positioned_args;
        exprs.push(MalType::List(varargs));
        Env::new(Some(c_env.env.clone()), binds, exprs)
    } else {
        ensure!(exprs.len() == binds.len(), "closure arguments not match params");
        Env::new(Some(c_env.env.clone()), binds, exprs)
    };

    eval(c_env.body.clone(), new_env)
}

fn call_closure(closure: &Closure, params: Vec<MalType>) -> Fallible<MalType> {
    let f = &closure.func;
    f(params, closure.c_env.clone())
}

fn quasiquote(ast: MalType) -> MalType {
    if !is_pair(&ast) {
        return MalType::List(vec![MalType::Symbol("quote".to_string()), ast]);
    }

    let mut list = ast.get_items();
    let first = list.remove(0);
    if first.is_symbol() && first.get_symbol_ref() == "unquote" {
        return list.remove(0);
    }

    if is_pair(&first) {
        let mut list_of_first = first.clone().get_items();
        let first_of_first = list_of_first.remove(0);
        if first_of_first.is_symbol() && first_of_first.get_symbol_ref() == "splice-unquote" {
            let ret = vec![MalType::Symbol("concat".to_string()), list_of_first.remove(0), quasiquote(MalType::Vec(list))];
            return MalType::List(ret);
        }
    }

    let l = vec![MalType::Symbol("cons".to_string()), quasiquote(first), quasiquote(MalType::Vec(list))];

    MalType::List(l)
}

fn is_pair(param: &MalType) -> bool {
    param.is_collection() && param.get_items_ref().len() > 0
}

fn is_macro_call(ast: &MalType, env: Env) -> bool {
    if ast.did_collection_have_leading_symbol() {
        let items = ast.get_items_ref();
        let symbol = env.get(items[0].get_symbol_ref());
        return symbol.map(|f| f.is_closure() && f.is_macro_closure()) == Some(true);
    }

    false
}

fn macroexpand(mut ast: MalType, env: Env) -> Fallible<MalType> {
    while is_macro_call(&ast, env.clone()) {
        let mut items = ast.get_items();
        let first_el = items.remove(0);
        let func = env.get(first_el.get_symbol_ref()).expect("get macro func");
        ast = call_closure(&func.get_closure(), items)?;
    }
    Ok(ast)
}

fn read(line: &str) -> Fallible<MalType> {
    read_str(line)
}

fn eval(mut mal: MalType, mut env: Env) -> Fallible<MalType> {
    loop {
        if !mal.is_list() || mal.is_empty_list() {
            return eval_ast(mal, env.clone());
        }

        mal = macroexpand(mal, env.clone())?;
        if !mal.is_list() || mal.is_empty_list() {
            return eval_ast(mal, env.clone());
        }

        let mut list = mal.get_items();
        let first_mal = list.remove(0);

        if first_mal.is_symbol() {
            match first_mal.get_symbol_ref().as_ref() {
                "def!" => {
                    ensure!(list.len() == 2, "def! should have 2 params");
                    let symbol_key = list.remove(0).get_symbol();
                    let value = eval(list.remove(0), env.clone())?;
                    return Ok(env.set(symbol_key, value));
                }
                "let*" => {
                    ensure!(list.len() == 2, "let* should have 2 params");
                    let mut new_env = Env::new(Some(env.clone()), Vec::new(), Vec::new());
                    let mut binding_list = list.remove(0).get_items();
                    ensure!(binding_list.len() % 2 == 0, "def! binding list should have 2n params");
                    while binding_list.len() >= 2 {
                        let key = binding_list.remove(0).get_symbol();
                        let value = eval(binding_list.remove(0), new_env.clone())?;
                        new_env.set(key, value);
                    }
                    let formula = list.remove(0);
                    env = new_env;
                    mal = formula;
                    continue;
                }
                "do" => {
                    let last = list.pop();
                    for item in list {
                        let _ = eval(item, env.clone())?;
                    }
                    if let Some(last) = last {
                        mal = last;
                        continue;
                    }
                    return Ok(MalType::Nil);
                }
                "if" => {
                    ensure!(list.len() >= 2, "if should have at least 2 params");
                    ensure!(list.len() <= 3, "if should have at most 3 params");
                    let condition_expr = list.remove(0);
                    let then_clause = list.remove(0);
                    let condition = eval(condition_expr, env.clone())?;
                    return match condition {
                        MalType::Nil | MalType::Bool(false) => {
                            if list.len() > 0 {
                                let else_clause = list.remove(0);
                                mal = else_clause;
                                continue;
                            } else {
                                Ok(MalType::Nil)
                            }
                        }
                        _ => {
                            mal = then_clause;
                            continue;
                        }
                    };
                }
                "fn*" => {
                    ensure!(list.len() == 2, "fn* should have 2 params");
                    let c_env = ClosureEnv::new(
                        list.remove(0),
                        list.remove(0),
                        env.clone(),
                    );
                    return Ok(MalType::Closure(Closure::new(call, Some(c_env))));
                }
                "eval" => {
                    ensure!(list.len() == 1, "eval should have 1 params");
                    let ret = eval(list.remove(0), env.clone())?;
                    mal = ret;
                    env = env.root();
                    continue;
                }
                "swap!" => {
                    let mut params: Vec<MalType> = list.into_iter().map(|el| eval(el, env.clone())).collect::<Fallible<Vec<MalType>>>()?;
                    ensure!(params.len() >= 2, "swap! should have more than 2 params");
                    let atom = params.remove(0);
                    let func = params.remove(0);
                    ensure!(atom.is_atom(), "swap!'s first param should be of type atom");
                    ensure!(func.is_closure(), "swap!'s second param should be a func");

                    let old_mal = atom.get_atom();
                    params.insert(0, old_mal);
                    if let MalType::Atom(a) = atom {
                        let new_mal =  call_closure(&func.get_closure(), params)?;
                        let _ = a.replace(new_mal.clone());
                        return Ok(new_mal);
                    }

                    unreachable!()
                }
                "quote" => {
                    ensure!(list.len() == 1, "quote should have 1 param");
                    return Ok(list.remove(0));
                }
                "quasiquote" => {
                    mal = quasiquote(list.remove(0));
                    continue;
                }
                "defmacro!" => {
                    ensure!(list.len() == 2, "defmacro! should have 2 params");
                    let symbol_key = list.remove(0).get_symbol();
                    let mut value = eval(list.remove(0), env.clone())?;
                    ensure!(value.is_closure(), "defmacro!'s second param should evaluate to func");
                    value.set_is_macro();
                    return Ok(env.set(symbol_key, value));
                }
                "macroexpand" => {
                    return macroexpand(list.remove(0), env.clone());
                }
                "env" => {
                    println!("{:#?}", env);
                    return Ok(MalType::Nil);
                }
                _ => {}
            };
        };

        let new_first_mal = eval(first_mal, env.clone())?;
        return match new_first_mal {
            MalType::Closure(ref closure) => {
                let params = list.into_iter().map(|el| eval(el, env.clone())).collect::<Fallible<Vec<MalType>>>()?;
                call_closure(closure, params)
            }
            _ => {
                bail!("{:?} is not a function", new_first_mal)
            }
        };
    }
}

fn eval_ast(ast: MalType, env: Env) -> Fallible<MalType> {
    match ast {
        MalType::Symbol(s) => {
            return env.get(&s).map_or(Err(format_err!("'{}' not found", s)), |f| Ok(f.clone()));
        }
        MalType::List(list) => return Ok(MalType::List(list.into_iter().map(|el| eval(el, env.clone())).collect::<Fallible<Vec<MalType>>>()?)),
        MalType::Vec(list) => return Ok(MalType::Vec(list.into_iter().map(|el| eval(el, env.clone())).collect::<Fallible<Vec<MalType>>>()?)),
        MalType::Hashmap(list) => {
            let (keys, values): (Vec<(usize, MalType)>, Vec<(usize, MalType)>) =
                list.into_iter()
                    .enumerate()
                    .partition(|&(ref index, _)| index % 2 == 0);

            ensure!(keys.len() == values.len(), "not valid hashmap");
            let new_values: Vec<MalType> =
                values.into_iter()
                    .map(|(_, el)| eval(el, env.clone()))
                    .collect::<Fallible<Vec<MalType>>>()?;

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


fn print(s: &MalType) -> String {
    pr_str(s, true)
}

fn rep(s: &str, env: Env) -> Fallible<String> {
    let ret = Ok(print(&eval(read(s)?, env.clone())?));
//    println!("env: {}", env);
    return ret;
}

fn main() -> Fallible<()> {
    pretty_env_logger::init();

    let ns = Ns::new();
    let mut repl_env = Env::new(None, Vec::new(), Vec::new());
    for (k, v) in ns.map {
        repl_env.set(k, MalType::Closure(v));
    }

    let _ = rep("(def! not (fn* (a) (if a false true)))", repl_env.clone())?;
    let _ = rep(r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) ")")))))"#, repl_env.clone())?;
    let _ = rep(r#"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"#, repl_env.clone())?;
    let _ = rep(r#"(defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) `(let* (or_FIXME ~(first xs)) (if or_FIXME or_FIXME (or ~@(rest xs))))))))"#, repl_env.clone())?;

    let mut args: Vec<String> = env::args().collect();
    let _self_name = args.remove(0);

    let mut filename = None;
    if args.len() > 0 {
        filename = Some(args.remove(0));
        repl_env.set("*ARGV*".to_string(), MalType::List(args.into_iter().map(|e| MalType::String(e)).collect()));
    } else {
        repl_env.set("*ARGV*".to_string(), MalType::List(Vec::new()));
    }

    match filename {
        Some(filename) => {
            let _ = rep(format!(r#"(load-file "{}")"#, filename).as_ref(), repl_env)?;
        }
        None => {
            let mut rl = Editor::<()>::new();
            if rl.load_history(HIST_PATH).is_err() {
                println!("No previous history.")
            }
            loop {
                let line = rl.readline("user> ");
                match line {
                    Ok(line) => {
                        rl.add_history_entry(line.as_ref());
                        match rep(line.as_ref(), repl_env.clone()) {
                            Ok(s) => println!("{}", s),
                            Err(e) => {
                                let downcast = e.downcast::<CommentFoundError>();
                                match downcast {
                                    Ok(_e) => {
                                        continue;
                                    }
                                    Err(e) => {
                                        println!("{}", e)
                                    }
                                }
                            }
                        }
                    }
                    Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                        break;
                    }
                    Err(err) => {
                        println!("no comment");
                        return Err(err.into());
                    }
                }
            }

            rl.save_history(HIST_PATH)?;
        }
    };

    Ok(())
}
