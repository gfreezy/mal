#![feature(nll)]
extern crate core;
#[macro_use]
extern crate failure;
//#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate rs;
extern crate rustyline;

use failure::Fallible;
use rs::core::Ns;
use rs::env::Env;
use rs::error::CommentFoundError;
use rs::error::MalExceptionError;
use rs::printer::pr_str;
use rs::reader::read_str;
use rs::types::Closure;
use rs::types::ClosureEnv;
use rs::types::MalType;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::collections::LinkedList;

const HIST_PATH: &str = ".mal-history";

fn call_for_closure(mut params: LinkedList<MalType>, c_env: Option<Rc<ClosureEnv>>) -> Fallible<MalType> {
    ensure!(c_env.is_some(), "closure env should be available");
    let c_env = c_env.unwrap();
    let len = params.len();
    let mut binds = c_env.parameters.to_symbol_list();

    let idx = binds.iter().position(|e| *e == "&");

    let new_env = if let Some(idx) = idx {
        ensure!(binds.len() == idx + 2, "& must be followed by a param name");
        ensure!(len >= idx, "closure arguments not match params");

        // drop "&"
        binds.remove(idx);
        let varargs = params.split_off(idx);
        let mut exprs: Vec<MalType> = params.into_iter().collect();
        exprs.push(MalType::List(varargs, Box::new(MalType::Nil)));
        Env::new(Some(c_env.env.clone()), binds, exprs)
    } else {
        ensure!(
            len == binds.len(),
            "closure arguments not match params"
        );
        Env::new(Some(c_env.env.clone()), binds, params.into_iter().collect())
    };

    eval(c_env.body.clone(), new_env)
}

fn quasiquote(ast: MalType) -> MalType {
    if !is_pair(&ast) {
        return MalType::List(
            linked_list![MalType::Symbol("quote".to_string()), ast],
            Box::new(MalType::Nil),
        );
    }

    let mut list = ast.into_items();
    let first = list.pop_front().unwrap();
    if first.is_symbol() && first.to_symbol() == "unquote" {
        return list.pop_front().unwrap();
    }

    if is_pair(&first) {
        let mut list_of_first = first.clone().into_items();
        let first_of_first = list_of_first.pop_front().unwrap();
        if first_of_first.is_symbol() && first_of_first.to_symbol() == "splice-unquote" {
            let ret = linked_list![
                MalType::Symbol("concat".to_string()),
                list_of_first.pop_front().unwrap(),
                quasiquote(MalType::Vec(list, Box::new(MalType::Nil))),
            ];
            return MalType::List(ret, Box::new(MalType::Nil));
        }
    }

    let l = linked_list![
        MalType::Symbol("cons".to_string()),
        quasiquote(first),
        quasiquote(MalType::Vec(list, Box::new(MalType::Nil))),
    ];

    MalType::List(l, Box::new(MalType::Nil))
}

fn is_pair(param: &MalType) -> bool {
    param.is_collection() && !param.to_items().is_empty()
}

fn is_macro_call(ast: &MalType, env: &Env) -> bool {
    if ast.did_collection_have_leading_symbol() {
        let items = ast.to_items();
        let symbol = env.get(items.front().unwrap().to_symbol());
        return symbol.map(|f| f.is_closure() && f.is_macro_closure()) == Some(true);
    }

    false
}

fn macroexpand(mut ast: MalType, env: &Env) -> Fallible<MalType> {
    while is_macro_call(&ast, env) {
        let mut items = ast.into_items();
        let first_el = items.pop_front().unwrap();
        let func = env.get(first_el.to_symbol()).expect("get macro func");
        ast = func.into_closure().call(items)?;
    }
    Ok(ast)
}

fn read(line: &str) -> Fallible<MalType> {
    read_str(line)
}

fn eval(mut mal: MalType, mut env: Env) -> Fallible<MalType> {
    loop {
        if !mal.is_list() || mal.is_empty_list() {
            return eval_ast(mal, &env);
        }

        mal = macroexpand(mal, &env)?;
        if !mal.is_list() || mal.is_empty_list() {
            continue;
        }

        let mut list = mal.into_items();
        let first_mal = list.pop_front().unwrap();

        if first_mal.is_symbol() {
            match first_mal.to_symbol().as_ref() {
                "def!" => {
                    ensure!(list.len() == 2, "def! should have 2 params");
                    let symbol_key = list.pop_front().unwrap().into_symbol();
                    let value = eval(list.pop_front().unwrap(), env.clone())?;
                    env.set(symbol_key, value.clone());
                    return Ok(value);
                }
                "let*" => {
                    ensure!(list.len() == 2, "let* should have 2 params");
                    let mut new_env = Env::new(Some(env.clone()), Vec::new(), Vec::new());
                    let mut binding_list = list.pop_front().unwrap().into_items();
                    ensure!(
                        binding_list.len() % 2 == 0,
                        "def! binding list should have 2n params"
                    );
                    while binding_list.len() >= 2 {
                        let key = binding_list.pop_front().unwrap().into_symbol();
                        let value = eval(binding_list.pop_front().unwrap(), new_env.clone())?;
                        new_env.set(key, value);
                    }
                    let formula = list.pop_front().unwrap();
                    env = new_env;
                    mal = formula;
                    continue;
                }
                "do" => {
                    let last = list.pop_back();
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
                    let condition_expr = list.pop_front().unwrap();
                    let then_clause = list.pop_front().unwrap();
                    let condition = eval(condition_expr, env.clone())?;
                    match condition {
                        MalType::Nil | MalType::Bool(false) => {
                            if !list.is_empty() {
                                let else_clause = list.pop_front().unwrap();
                                mal = else_clause;
                                continue;
                            } else {
                                return Ok(MalType::Nil);
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
                    let c_env = ClosureEnv::new(list.pop_front().unwrap(), list.pop_front().unwrap(), env.clone());
                    return Ok(MalType::Closure(
                        Closure::new(call_for_closure, Some(c_env)),
                        Box::new(MalType::Nil),
                    ));
                }
                "eval" => {
                    ensure!(list.len() == 1, "eval should have 1 params");
                    let ret = eval(list.pop_front().unwrap(), env.clone())?;
                    mal = ret;
                    env = env.root();
                    continue;
                }
                "quote" => {
                    ensure!(list.len() == 1, "quote should have 1 param");
                    return Ok(list.pop_front().unwrap());
                }
                "quasiquote" => {
                    mal = quasiquote(list.pop_front().unwrap());
                    continue;
                }
                "defmacro!" => {
                    ensure!(list.len() == 2, "defmacro! should have 2 params");
                    let symbol_key = list.pop_front().unwrap().into_symbol();
                    let mut value = eval(list.pop_front().unwrap(), env.clone())?;
                    ensure!(
                        value.is_closure(),
                        "defmacro!'s second param should evaluate to func"
                    );
                    value.set_is_macro();
                    env.set(symbol_key, value.clone());
                    return Ok(value);
                }
                "macroexpand" => {
                    return macroexpand(list.pop_front().unwrap(), &env);
                }
                "try*" => {
                    ensure!(list.len() == 2, "try* should have 2 params");
                    let stmt = list.pop_front().unwrap();
                    let catch = list.pop_front().unwrap();
                    ensure!(
                        catch.get_first_symbol().map(|s| s.to_symbol())
                            == Some(&"catch*".to_string()),
                        "invalid syntax"
                    );
                    let mut catch_clause = catch.into_items();
                    // remove "catch*" symbol
                    catch_clause.pop_front().unwrap();
                    ensure!(catch_clause.len() == 2, "catch* should have 2 params");

                    let exception = match eval(stmt, env.clone()) {
                        Ok(ast) => return Ok(ast),
                        Err(e) => {
                            let downcast = e.downcast::<MalExceptionError>();
                            match downcast {
                                Ok(MalExceptionError(s)) => read_str(&s)?,
                                Err(e) => MalType::String(format!("{}", e)),
                            }
                        }
                    };

                    let variable_name_mal = catch_clause.pop_front().unwrap();
                    ensure!(
                        variable_name_mal.is_symbol(),
                        "catch* first param should be symbol"
                    );
                    let variable_name = variable_name_mal.into_symbol();
                    let new_env = Env::new(Some(env.clone()), vec![variable_name], vec![exception]);
                    let catch_stmt = catch_clause.pop_front().unwrap();
                    mal = catch_stmt;
                    env = new_env;
                    continue;
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
            MalType::Closure(ref closure, ..) => {
                let mut params: LinkedList<MalType> = list
                    .into_iter()
                    .map(|el| eval(el, env.clone()))
                    .collect::<Fallible<LinkedList<MalType>>>()?;

                let c_env = closure.c_env.clone();
                if let Some(c_env) = c_env {
                    let len = params.len();
                    let mut binds = c_env.parameters.to_symbol_list();

                    let idx = binds.iter().position(|e| *e == "&");

                    let new_env = if let Some(idx) = idx {
                        ensure!(binds.len() == idx + 2, "& must be followed by a param name");
                        ensure!(len >= idx, "closure arguments not match params");

                        // drop "&"
                        binds.remove(idx);
                        let varargs = params.split_off(idx);
                        let mut exprs: Vec<MalType> = params.into_iter().collect();
                        exprs.push(MalType::List(varargs, Box::new(MalType::Nil)));
                        Env::new(Some(c_env.env.clone()), binds, exprs)
                    } else {
                        ensure!(
                            len == binds.len(),
                            "closure arguments not match params"
                        );
                        Env::new(Some(c_env.env.clone()), binds, params.into_iter().collect())
                    };

                    // eval(c_env.body.clone(), new_env)
                    mal = c_env.body.clone();
                    env = new_env;
                    continue;
                } else {
                    closure.call(params)
                }
            }
            _ => bail!("{:?} is not a function", new_first_mal),
        };
    }
}

fn eval_ast(ast: MalType, env: &Env) -> Fallible<MalType> {
    match ast {
        MalType::Symbol(s) => env
            .get(&s)
            .map_or(Err(format_err!("'{}' not found", s)), |f| Ok(f)),
        MalType::List(list, ..) => Ok(MalType::List(
            list.into_iter()
                .map(|el| eval(el, env.clone()))
                .collect::<Fallible<LinkedList<MalType>>>()?,
            Box::new(MalType::Nil),
        )),
        MalType::Vec(list, ..) => Ok(MalType::Vec(
            list.into_iter()
                .map(|el| eval(el, env.clone()))
                .collect::<Fallible<LinkedList<MalType>>>()?,
            Box::new(MalType::Nil),
        )),
        MalType::Hashmap(mapping, ..) => {
            let mut new_mapping = HashMap::new();
            for (k, v) in mapping {
                new_mapping.insert(k, eval(v, env.clone())?);
            }
            Ok(MalType::Hashmap(new_mapping, Box::new(MalType::Nil)))
        }
        _ => Ok(ast),
    }
}

fn print(s: &MalType) -> String {
    pr_str(s, true)
}

fn rep(s: &str, env: &Env) -> Fallible<String> {
    Ok(print(&eval(read(s)?, env.clone())?))
}

fn main() -> Fallible<()> {
    pretty_env_logger::init();

    let ns = Ns::new();
    let mut repl_env = Env::new(None, Vec::new(), Vec::new());
    for (k, v) in ns.map {
        repl_env.set(k, MalType::Closure(v, Box::new(MalType::Nil)));
    }

    repl_env.set(
        "*host-language*".to_string(),
        MalType::String("mal".to_string()),
    );
    let _ = rep("(def! not (fn* (a) (if a false true)))", &repl_env)?;
    let _ = rep(
        r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) ")")))))"#,
        &repl_env,
    )?;
    let _ = rep(r#"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"#, &repl_env)?;
    let _ = rep(r#"(defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) `(let* (or_FIXME ~(first xs)) (if or_FIXME or_FIXME (or ~@(rest xs))))))))"#, &repl_env)?;
    let _ = rep(r#"(do (def! *gensym-counter* (atom 0))
    (def! gensym (fn* [] (symbol (str "G__" (swap! *gensym-counter* (fn* [x] (+ 1 x)))))))
    (defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) (let* (condvar (gensym)) `(let* (~condvar ~(first xs)) (if ~condvar ~condvar (or ~@(rest xs))))))))))
    "#, &repl_env)?;
    let mut args: Vec<String> = env::args().collect();
    let _self_name = args.remove(0);

    let mut filename = None;
    if !args.is_empty() {
        filename = Some(args.remove(0));
        repl_env.set(
            "*ARGV*".to_string(),
            MalType::List(
                args.into_iter().map(MalType::String).collect(),
                Box::new(MalType::Nil),
            ),
        );
    } else {
        repl_env.set(
            "*ARGV*".to_string(),
            MalType::List(LinkedList::new(), Box::new(MalType::Nil)),
        );
    }

    match filename {
        Some(filename) => {
            let _ = rep(format!(r#"(load-file "{}")"#, filename).as_ref(), &repl_env)?;
        }
        None => {
            let mut rl = Editor::<()>::new();
            if rl.load_history(HIST_PATH).is_err() {
                println!("No previous history.")
            }
            let _ = rep(r#"(println (str "Mal [" *host-language* "]"))"#, &repl_env)?;

            loop {
                let line = rl.readline("user> ");
                match line {
                    Ok(line) => {
                        rl.add_history_entry(line.as_ref());
                        match rep(line.as_ref(), &repl_env) {
                            Ok(s) => println!("{}", s),
                            Err(e) => {
                                let downcast = e.downcast::<CommentFoundError>();
                                match downcast {
                                    Ok(_e) => {
                                        continue;
                                    }
                                    Err(e) => println!("{}", e),
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
