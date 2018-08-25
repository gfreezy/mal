extern crate failure;
extern crate rs;
extern crate rustyline;

use failure::Fallible;
use rs::printer::pr_str;
use rs::reader::read_str;
use rs::types::MalType;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn read(line: &str) -> Fallible<MalType> {
    read_str(line)
}

fn eval(s: &MalType) -> &MalType {
    s
}

fn print(s: &MalType) -> String {
    pr_str(s, true)
}

fn rep(s: &str) -> Fallible<String> {
    Ok(print(eval(&read(s)?)))
}

const HIST_PATH: &str = ".mal-history";

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
