extern crate failure;
extern crate rustyline;


use failure::Error;
use rustyline::Editor;
use rustyline::error::ReadlineError;


fn read(line: &str) -> &str {
    line
}


fn eval(s: &str) -> &str {
    s
}


fn print(s: &str) -> &str {
    s
}

fn rep(s: &str) -> &str {
    print(eval(read(s)))
}

const HIST_PATH: &str = ".mal-history";


fn main() -> Result<(), Error> {
    let mut rl = Editor::<()>::new();
    if rl.load_history(HIST_PATH).is_err() {
        println!("No previous history.")
    }

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                println!("{}", rep(line.as_ref()));
            },
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                break;
            },
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    rl.save_history(HIST_PATH)?;
    Ok(())
}
