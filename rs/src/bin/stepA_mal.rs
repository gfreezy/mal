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
use rs::env::env_get;
use rs::env::env_new;
use rs::env::env_root;
use rs::env::env_set;
use rs::env::Env;
use rs::error::CommentFoundError;
use rs::error::MalExceptionError;
use rs::printer::pr_str;
use rs::reader::read_str;
use rs::types::Closure;
use rs::types::ClosureEnv;
use rs::types::{MalType, InnerMalType};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::env;
use std::rc::Rc;

const HIST_PATH: &str = ".mal-history";

fn call_for_closure(
    mut params: LinkedList<MalType>,
    c_env: Option<ClosureEnv>,
) -> Fallible<MalType> {
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
        exprs.push(new_mal!(List(varargs, new_mal!(Nil))));
        env_new(Some(c_env.env.clone()), binds, exprs)
    } else {
        ensure!(len == binds.len(), "closure arguments not match params");
        env_new(Some(c_env.env.clone()), binds, params.into_iter().collect())
    };

    eval(c_env.body.clone(), new_env)
}

fn quasiquote(ast: MalType) -> MalType {
    if !is_pair(&ast) {
        return new_mal!(List(
            linked_list![new_mal!(Symbol("quote".to_string())), ast],
            new_mal!(Nil)
        ));
    }

    let mut list = ast.to_items();
    let first = list.pop_front().unwrap();
    if first.is_symbol() && first.to_symbol() == "unquote" {
        return list.pop_front().unwrap();
    }

    if is_pair(&first) {
        let mut list_of_first = first.clone().to_items();
        let first_of_first = list_of_first.pop_front().unwrap();
        if first_of_first.is_symbol() && first_of_first.to_symbol() == "splice-unquote" {
            let ret = linked_list![
                new_mal!(Symbol("concat".to_string())),
                list_of_first.pop_front().unwrap(),
                quasiquote(new_mal!(Vec(list, new_mal!(Nil)))),
            ];
            return new_mal!(List(ret, new_mal!(Nil)));
        }
    }

    let l = linked_list![
        new_mal!(Symbol("cons".to_string())),
        quasiquote(first),
        quasiquote(new_mal!(Vec(list, new_mal!(Nil)))),
    ];

    new_mal!(List(l, new_mal!(Nil)))
}

fn is_pair(param: &MalType) -> bool {
    param.is_collection() && !param.to_items().is_empty()
}

fn is_macro_call(ast: &MalType, env: &Env) -> bool {
    if ast.did_collection_have_leading_symbol() {
        let items = ast.to_items();
        let symbol = env_get(env.clone(), &items.front().unwrap().to_symbol());
        symbol.map(|f| f.is_closure() && f.is_macro_closure()) == Some(true)
    } else {
        false
    }
}

fn macroexpand(mut ast: MalType, env: &Env) -> Fallible<MalType> {
    while is_macro_call(&ast, env) {
        let mut items = ast.to_items();
        let first_el = items.pop_front().unwrap();
        let func = env_get(env.clone(), &first_el.to_symbol()).expect("get macro func");
        ast = func.to_closure().call(items)?;
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

        let mut list = mal.to_items();
        let first_mal = list.pop_front().unwrap();

        if first_mal.is_symbol() {
            match first_mal.to_symbol().as_ref() {
                "def!" => {
                    ensure!(list.len() == 2, "def! should have 2 params");
                    let symbol_key = list.pop_front().unwrap().to_symbol();
                    let value = eval(list.pop_front().unwrap(), env.clone())?;
                    env_set(env.clone(), symbol_key, value.clone());
                    return Ok(value);
                }
                "let*" => {
                    ensure!(list.len() == 2, "let* should have 2 params");
                    let new_env = env_new(Some(env.clone()), Vec::new(), Vec::new());
                    let mut binding_list = list.pop_front().unwrap().to_items();
                    ensure!(
                        binding_list.len() % 2 == 0,
                        "def! binding list should have 2n params"
                    );
                    while binding_list.len() >= 2 {
                        let key = binding_list.pop_front().unwrap().to_symbol();
                        let value = eval(binding_list.pop_front().unwrap(), new_env.clone())?;
                        env_set(new_env.clone(), key, value);
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
                    return Ok(new_mal!(Nil));
                }
                "if" => {
                    ensure!(list.len() >= 2, "if should have at least 2 params");
                    ensure!(list.len() <= 3, "if should have at most 3 params");
                    let condition_expr = list.pop_front().unwrap();
                    let then_clause = list.pop_front().unwrap();
                    let condition = eval(condition_expr, env.clone())?;
                    match &*condition {
                        InnerMalType::Nil | InnerMalType::Bool(false) => {
                            if !list.is_empty() {
                                let else_clause = list.pop_front().unwrap();
                                mal = else_clause;
                                continue;
                            } else {
                                return Ok(new_mal!(Nil));
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
                        list.pop_front().unwrap(),
                        list.pop_front().unwrap(),
                        env.clone(),
                    );
                    return Ok(new_mal!(Closure(
                        Closure::new(call_for_closure, Some(c_env)),
                        new_mal!(Nil)
                    )));
                }
                "eval" => {
                    ensure!(list.len() == 1, "eval should have 1 params");
                    let ret = eval(list.pop_front().unwrap(), env.clone())?;
                    mal = ret;
                    env = env_root(env);
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
                    let symbol_key = list.pop_front().unwrap().to_symbol();
                    let mut value = eval(list.pop_front().unwrap(), env.clone())?;
                    ensure!(
                        value.is_closure(),
                        "defmacro!'s second param should evaluate to func"
                    );
                    let new_value = Rc::make_mut(&mut value);
                    new_value.set_is_macro();
                    env_set(env.clone(), symbol_key, value.clone());
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
                            == Some("catch*".to_string()),
                        "invalid syntax"
                    );
                    let mut catch_clause = catch.to_items();
                    // remove "catch*" symbol
                    catch_clause.pop_front().unwrap();
                    ensure!(catch_clause.len() == 2, "catch* should have 2 params");

                    let exception = match eval(stmt, env.clone()) {
                        Ok(ast) => return Ok(ast),
                        Err(e) => {
                            let downcast = e.downcast::<MalExceptionError>();
                            match downcast {
                                Ok(MalExceptionError(s)) => read_str(&s)?,
                                Err(e) => new_mal!(String(format!("{}", e))),
                            }
                        }
                    };

                    let variable_name_mal = catch_clause.pop_front().unwrap();
                    ensure!(
                        variable_name_mal.is_symbol(),
                        "catch* first param should be symbol"
                    );
                    let variable_name = variable_name_mal.to_symbol();
                    let new_env = env_new(Some(env.clone()), vec![variable_name], vec![exception]);
                    let catch_stmt = catch_clause.pop_front().unwrap();
                    mal = catch_stmt;
                    env = new_env;
                    continue;
                }
                "env" => {
                    println!("{:#?}", env);
                    return Ok(new_mal!(Nil));
                }
                _ => {}
            };
        };

        let new_first_mal = eval(first_mal, env.clone())?;
        return match &*new_first_mal {
            InnerMalType::Closure(closure, ..) => {
                let mut params: LinkedList<MalType> = LinkedList::new();
                for el in list {
                    params.push_back(eval(el, env.clone())?);
                }

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
                        exprs.push(new_mal!(List(varargs, new_mal!(Nil))));
                        env_new(Some(c_env.env.clone()), binds, exprs)
                    } else {
                        ensure!(len == binds.len(), "closure arguments not match params");
                        env_new(Some(c_env.env.clone()), binds, params.into_iter().collect())
                    };

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
    let ast = Rc::try_unwrap(ast).unwrap_or_else(|s| (*s).clone());
    match ast {
        InnerMalType::Symbol(s) => {
            env_get(env.clone(), &s).map_or(Err(format_err!("'{}' not found", s)), Ok)
        }
        InnerMalType::List(list, ..) => {
            let mut new_l = LinkedList::new();
            for el in list {
                new_l.push_back(eval(el, env.clone())?);
            }
            Ok(new_mal!(List(new_l, new_mal!(Nil))))
        }
        InnerMalType::Vec(list, ..) => {
            let mut new_l = LinkedList::new();
            for el in list {
                new_l.push_back(eval(el, env.clone())?);
            }
            Ok(new_mal!(Vec(new_l, new_mal!(Nil))))
        }
        InnerMalType::Hashmap(mapping, ..) => {
            let mut new_mapping = HashMap::new();
            for (k, v) in mapping {
                new_mapping.insert(k, eval(v, env.clone())?);
            }
            Ok(new_mal!(Hashmap(new_mapping, new_mal!(Nil))))
        }
        _ => Ok(Rc::new(ast)),
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
    let repl_env = env_new(None, Vec::new(), Vec::new());
    for (k, v) in ns.map {
        env_set(
            repl_env.clone(),
            k,
            new_mal!(Closure(v, new_mal!(Nil))),
        );
    }

    env_set(
        repl_env.clone(),
        "*host-language*".to_string(),
        new_mal!(String("mal".to_string())),
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
        env_set(
            repl_env.clone(),
            "*ARGV*".to_string(),
            new_mal!(List(
                args.into_iter().map(|s|new_mal!(String(s))).collect(),
                new_mal!(Nil)
            )),
        );
    } else {
        env_set(
            repl_env.clone(),
            "*ARGV*".to_string(),
            new_mal!(List(LinkedList::new(), new_mal!(Nil))),
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
