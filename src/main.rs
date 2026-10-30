use crate::{error::ProgramError, parser::Parser, processes::kill_processes};
use std::fmt::Write;

mod common;
mod error;
mod lexer;
mod parser;
mod processes;

fn unwrap_or_print<T>(res: Result<T, ProgramError>, source: &str) -> T {
    match res {
        Ok(ok) => ok,
        Err(e) => {
            e.show_error(source);
            std::process::exit(1)
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: String = {
        let raw = std::env::args().skip(1);
        if raw.len() == 0 {
            println!("Se esperaba 1 o mas programas.");
            std::process::exit(0);
        }
        let mut new = String::new();
        for arg in raw {
            write!(new, "{} ", arg)?;
        }
        new
    };
    let procs = {
        let mut parser = unwrap_or_print(Parser::from_source(&args), &args);
        unwrap_or_print(parser.get_processes(), &args)
    };
    let not_killed = kill_processes(procs);

    for program in &not_killed.names {
        println!("Proceso {program} no pudo ser papeado.")
    }
    for pid in &not_killed.pids {
        println!("Proceso con pid {pid} no pudo ser papeado.")
    }

    Ok(())
}
