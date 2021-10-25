#![allow(dead_code)]
mod plot;
mod structs;
mod structure;
use plot::plot;
use structure::*;

use std::io::{stdin, stdout, Write};
use std::path::Path;

fn main() {
    let structure = get_structure(Path::new("../git-cuttle"), &[".vscode", "target", ".git"]);
    println!("{:?}", structure);
    if let Some(item) = structure {
        plot(item, "plot.svg");
    }

    let mut s = String::new();
    print!(">");
    let _ = stdout().flush();
    stdin().read_line(&mut s).expect("Incorrect input");
    let structure = get_structure(Path::new("../pdbtbx"), &[".vscode", "target", ".git"]);
    if let Some(item) = structure {
        plot(item, "plot.svg");
    }
}
