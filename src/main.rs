#![allow(unused_imports)]
use clap::{parser, Command, Arg, arg};
use anyhow::{Context, Result};

fn githydra() -> Command {

    Command::new("githydra")
        .about("GIthydra")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("add_account")
                .about("Add a new account to your Githydra directory")
                .arg_required_else_help(true)
                .arg(Arg::new("verbose").short('v'))
                .arg(Arg::new("help").short('h'))
                .arg(
                    Arg::new("dirname")
                )
                .arg(
                    Arg::new("ssh-config-path")
                )
                .arg(
                    Arg::new("ssh-config-dir")
                )
                .arg(
                    Arg::new("ssh-alias")
                )
                .arg(
                    Arg::new("ssh-key-name")
                )
                .arg(
                    Arg::new("ssh-privkey-path")
                )
                .arg(
                    Arg::new("template_script-path")
                )
                .arg(
                    Arg::new("git-hooks-dir")
                )
        )


}

fn main() -> Result<()> {

    let matches = githydra().get_matches();

    match matches.subcommand() {
        Some(("add_account", _sub_matches)) => {

            println!("add_account called");
        }
        _ => unreachable!()
    }

    Ok(())
}
