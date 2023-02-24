use clap::CommandFactory;
use clap_complete::{generate_to, shells};
use std::env;
use std::io::Error;

use wleave::cli_opt::Args;

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Args::command();

    println!(
        "Bash completions generated: {:?}",
        generate_to(shells::Bash, &mut cmd, "wleave", &outdir)?
    );

    println!(
        "Zsh completions generated: {:?}",
        generate_to(shells::Zsh, &mut cmd, "wleave", &outdir)?
    );

    println!(
        "Fish completions generated: {:?}",
        generate_to(shells::Fish, &mut cmd, "wleave", &outdir)?
    );

    Ok(())
}
