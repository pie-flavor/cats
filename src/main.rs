// In a real project this would have unit and integration tests.

#![deny(missing_debug_implementations, rust_2018_idioms)]

#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate rusqlite;
#[macro_use]
extern crate prettytable;

use crate::args::{Args, Cmd};
use anyhow::Result;
use rusqlite::Connection;
use std::process;
use structopt::StructOpt;

mod args;
mod cmds;
mod migrations;

fn main() {
    match main_() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(-1);
        }
    }
}

fn main_() -> Result<()> {
    use Cmd::*;
    const PATH: &str = "cat_registry.db";
    let conn = Connection::open(PATH)?;
    migrations::migration1(&conn)?;
    let Args { cmd, json } = Args::from_args();
    let (a, f, g, u, d);
    let result: &dyn Printable = match cmd {
        Add { cmd } => {
            a = cmds::add(&conn, cmd)?;
            &a
        }
        Delete { id } => {
            d = cmds::delete(&conn, id)?;
            &d
        }
        Find { cmd } => {
            f = cmds::find(&conn, cmd)?;
            &f
        }
        Get { id } => {
            g = cmds::get(&conn, &id)?;
            &g
        }
        Update { cmd } => {
            u = cmds::update(&conn, cmd)?;
            &u
        }
    };
    if json {
        result.print_json();
    } else if atty::is(atty::Stream::Stdout) {
        result.print_display();
    } else {
        result.print_plain();
    }
    Ok(())
}

trait Printable {
    fn print_display(&self);
    fn print_plain(&self);
    fn print_json(&self);
}
