#![allow(unused_imports)]
use clap::{parser, Command, Arg, arg, ArgMatches};
use anyhow::{Context, Result};
use githydra::error::{new_gh_err, GitHydraError as GHErr};
use std::borrow::Cow;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::{Stdio};

fn githydra() -> Command {

    Command::new("githydra")
        .about("Githydra")
        // .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("add_account")
                .about("Add a new account to your Githydra directory")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("username")
                        .required(true)
                        .index(1)
                        .help("Username for the new githydra user")
                )
                .arg(
                    Arg::new("dirname")
                        .required(false)
                        // .last(true) requires usage of "--dirname", and disallows
                        // positional I think, which is what I want
                        .last(true)
                        .long("dirname")
                        .help("Pass a specific name to use for this accounts directory, if left blank the directory will be named the same as `username`")
                )
                .arg(
                    Arg::new("ssh-config-path")
                        .last(true)
                        .help("Path to SSH config file. If no file exists at this path, a new SSH config file will be created there. Default location checked (and where the new config is created if none exists) is `/home/username/.ssh/config`")
                )
                .arg(
                    Arg::new("ssh-config-dir")
                        .help("Pass a custom location to for the new SSH config file to be created at. If left blank the default is `./ssh/config`")
                )
                .arg(
                    Arg::new("ssh-alias")
                        .help("Pass a specific SSH alias to use, if blank the default will be the same as CoolName param")
                )
                .arg(
                    Arg::new("ssh-key-name")
                        .help("Name for the newly generated SSH key. Default is `id_rsa_` followed by the CoolName param.")
                )
                .arg(
                    Arg::new("ssh-privkey-path")
                        .help("Path to SSH private key to use for this GitHydra directory. This is only used to update SSH config file alias.")
                )
                .arg(
                    Arg::new("ssh-email")
                        .help("Email to use when creating SSH key")
                )
                .arg(
                    Arg::new("template-script-path")
                        .help("Custom path to template path script generated with GitHydra in a previous use. Default is ./home/user/githydra/setgituser.js I THINK not sure verify")
                )
                .arg(
                    Arg::new("git-hooks-dir")
                        .help("Custom path to `git_templates/hooks`, otherwise some default")
                )
        )
}


fn main() -> Result<(), GHErr> {

    let matches = githydra().get_matches();

    match matches.subcommand() {
        Some(("add_account", sub_matches)) => {
            verify_installed(ToVerify::SSH_KEYGEN)?;
            verify_installed(ToVerify::GIT)?;

            handle_add_account(sub_matches)?;

            println!("add_account called");
        }
        _ => unreachable!()
    }

    Ok(())
}

fn handle_add_account(sub_matches: &ArgMatches) -> Result<(), GHErr> {

    // Safe to unwrap since "username" is required
    let github_username = sub_matches.get_one::<String>("username").unwrap();

    let whoami = {
        let whoami_output = std::process::Command::new("whoami")
            .output()
            .map_err(|e| GHErr::GenErr { error: format!("Error: {}",e) })?;
        let untrimmed = String::from_utf8(whoami_output.stdout)
            .map_err(|e| GHErr::GenErr { error: format!("Error: {}",e) })?;
        //strip newline
        Ok(untrimmed.trim().to_owned())
    }?;

    // user provided ssh key path OR path to one that's generated in this logic
    let ssh_privkey_path = match sub_matches.get_one::<String>("ssh-privkey-path") {
        Some(path) => Ok(path.to_owned()),
        None => {
            // Pull "ssh-key-gen" flag or use basic default{
            let ssh_key_name = match sub_matches.get_one::<String>("ssh-key-name") {
                Some(name) => name.to_owned(),
                None => format!("id_rsa_{}", github_username)
            };
            let ssh_key_path = format!("/home/{}/.ssh/{}", whoami, ssh_key_name);
            let comment = format!("For github account {}", github_username);

            // use ssh_key_name to create new SSH key, return the path to it
            // let output_sshkeygen = std::process::Command::new("ssh-keygen")
            //     .stdin(Stdio::piped())
            //     .stdout(Stdio::piped())
            //     .stderr(Stdio::piped())
            //     .args(["-t", "rsa", "-b", "4096", "-C", &comment, "-N", r#""""#, "-f", &ssh_key_path])
            //     .output()
            //     .map_err(|_e| new_gh_err("error"))?;
            // println!("output: {:#?}", output_sshkeygen);
            Ok(ssh_key_path)
            // TODO: Probably need to handle the "Key already exists" case here
            // if let Ok(mut child) = output_sshkeygen {
            //     if let Some(mut stdin) = child.stdin.take() {
            //         stdin.write_all("Y"/"N" as_bytes())
            //     } else {
            //         return Err(new_gh_err("No stdin available generating SSH key"));
            //     };
            //     let z = child.wait_with_output().map_err(|_e| new_gh_err("Error waiting for output"))?;
            //     println!("Output after writing: {:#?}", z);
            // } else {
            //     Err(GHErr::GenErr{error: String::from("Problem generating new SSH key")})
            // }
        }
    }?;


    // ------------------------------------
    // Creating a new template script if needed

    let template_script_path = sub_matches.get_one::<String>("template-script-path")
        .map_or(format!("/home/{}/githydra/test_script.js", whoami), |v| v.to_owned());

    match File::options().read(true).append(true).open(&template_script_path) {
        // If file exists already, do nothing
        Ok(_) => Ok(()),
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    // If file doesn't exist create it
                    let mut handle = File::create_new(&template_script_path)
                        .map_err(|_| new_gh_err("Error creating template script"))?;
                    // And write it
                    handle.write_all(&template_script_starter())
                        .map_err(|_e| new_gh_err("Error writing to new script"))
                },
                // If file exists already do nothing
                _ => Ok(())
            }
        }
    }?;

    //--------------------------------
    // Generate new lookup.json or add to it

    let mut lookup_file_handle = match File::options().read(true).append(true).open(format!("/home/{}/githydra/lookup.json", whoami)) {
        Ok(file) => Ok(file),
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    File::create_new(format!("/home/{}/githydra/lookup.json",whoami))
                            .map_err(|_| new_gh_err("Error creating lookup.json"))
                },
                _ => Err(new_gh_err("Error creating lookup.json"))
            }
        }
    }?;
    // How to write will depend on if it exists or not

    //--------------------------------
    // Generate git hook scripts if needed



    //--------------------------------
    // SSH Config stuff
    //
    // Verify `~/.ssh/config` exists or can be created
    // -- Check if a config exists first
    // - sshconfig is going to be the path to check if config exists
    let sshconfigpath: String = {
        if let Some(configpath) = sub_matches.get_one::<String>("ssh-config-path") {
            configpath.to_owned()
        } else {
            // format!("/home/{}/.ssh/config",username)
            format!("/home/{}/githydra/test.md", whoami)
        }
    };

    // If File::open(sshconfig) doesn't error
    // -- Keep the file handle so I can write to it
    // If File::open(sshconfig) does error
    // -- Create a new ssh config file and keep handle to it
    let mut ssh_config_file = match File::options().read(true).append(true).open(&sshconfigpath) {
        // Opening file worked fine, return handle
        Ok(file) => Ok(file),
        // Check what the error was opening file
        Err(e) => {
            match e.kind() {
                // File wasn't found, create new file at path
                std::io::ErrorKind::NotFound => {
                        File::create_new(&sshconfigpath)
                            .map_err(|_| GHErr::GenErr { error: format!("Problem creating new ssh config file at path: {}", sshconfigpath) })
                }
                // Any other error opening file
                _ => Err(GHErr::GenErr { error: format!("Problem opening ssh config file at path {}", sshconfigpath) } )
            }
        }
    }?;

    let ssh_alias = sub_matches
        .get_one::<String>("ssh-alias")
        .or(Some(github_username))
        .unwrap();

    // Using a testfile to make sure the ssh-config entries format correctly
    ssh_config_file.write_all(
        &serialize_ssh_config_entry(
            ssh_alias,
            &ssh_privkey_path
            // "~/.ssh/id_rsa_grokkt"
        )
    ).unwrap();


    // -----------------
    //
    // Step 5
    // - Making username directory
    let dir_name = sub_matches.get_one::<String>("dirname")
        .map_or(format!("/home/{}/{}", whoami, github_username), |v| {
            format!("/home/{}/{}", whoami, v)
        });

    fs::create_dir(dir_name)
        .map_err(|e| new_gh_err(format!("Error creating directory: {:#?}", e)))?;       




    // Script stuff
    // Ok so I need to generate a setgituser.js and lookup.json if they don't already
    // exist (if this is the first time the user calls add-account
    // -- Generating setgituser.js is done
    // -- Need to generate lookup.json

    Ok(())
}

fn serialize_ssh_config_entry(hostname: &str, privkeypath: &str) -> Vec<u8> {
    let entry = format!("# {} github\nHost {}\n        HostName github.com\n        User git\n        IdentityFile {}\n\n", hostname, hostname, privkeypath);
    let z = entry.as_bytes();
    z.to_vec()
}

//------

pub enum ToVerify {
    GIT,
    SSH_KEYGEN,
}
fn verify_installed(prog: ToVerify) -> Result<(), GHErr> {
    match prog {
        ToVerify::GIT => {
            let gitver = std::process::Command::new("which")
                .arg("git")
                .output()
                .map_err(|e| GHErr::GenErr {error: format!("Error: {}", e.to_string())})?;
            if gitver.stdout.len() == 0 {
                return Err(GHErr::GenErr {error: String::from("git must be installed")})
            }
        },
        ToVerify::SSH_KEYGEN => {
            // ver.stdout ends up being path if it's installed, like
            // "/usr/bin/ssh-keygen\n"
            // ver.stdout is blank if ssh-keygen isn't installed
            let gitver = std::process::Command::new("which")
                .arg("ssh-keygen")
                .output()
                .map_err(|e| GHErr::GenErr {error: format!("Error: {}", e.to_string())})?;
            if gitver.stdout.len() == 0 {
                return Err(GHErr::GenErr {error: String::from("ssh-keygen must be installed")})
            }
        }
    }
    Ok(())
}


//----

// Escape {} with doubles, so "{{}}" gives "{}"
fn template_script_starter() -> Vec<u8> {
    let imports = r#"const util = require('node:util');
const exec = util.promisify(require('node:child_process').exec);
const fs = require('node:fs/promises');

"#;

    // name: whoami or repo_root
    let temp_exec = |name: String, exec_cmd: String, err_msg: String| -> String {
        format!(r#"
const get_{} = async () => {{
    let {} = await exec('{}');
    if ({}.stderr) {{
        console.error(`{}`);
        process.exit(1);
    }}
    return ({}.stdout.indexOf('\n') != -1) ? {}.stdout.substring(0, {}.stdout.indexOf('\n')) : {}.stdout
}};

"#, name, name, exec_cmd, name, err_msg, name, name, name, name)};


    let get_lookup = r#"
const get_lookup = async () => {
    let f = await fs.readFile('./lookup.json', {encoding: 'utf8'});
    let obj = JSON.parse(f);
    return obj
};

"#;

    let iife = r#"
(async () => {

	const username = await get_whoami();
	const repo_root = await get_repo_root();
	const lookup = await get_lookup();

	for (let x = 0; x < lookup.length; x++) {
		// If the current repo path starts with a known lookup entry, set git config.email to corresp email
		if (repo_root.startsWith(`/home/${username}/${lookup[x].dir}`)) {
			const res = await exec (`git config user.email "${lookup[x].email}"`);
			if (res.stderr) {
				console.error(`Error setting git config user.email | ${res.stderr}`);
				process.exit(1);
			}
			const res2 = await exec (`git config user.name "${lookup[x].dir}"`);
			if (res2.stderr) {
				console.error(`Error setting git config user.name | ${res.stderr}`);
				process.exit(1);
			}
			console.log(`Git config user.email updated to ${lookup[x].email}`);
			console.log(`Git config user.name updated to ${lookup[x].dir}`);
			process.exit(0);
		}
	}

	console.log(`Directory doesnt match any known entries. Leaving git config email as default`);
	process.exit(0);
})();

"#;

    let fin = format!("{}{}{}{}{}", 
        imports,
        temp_exec(
            "whoami".to_string(),
            "whoami".to_string(),
            "Error getting current user via whoami".to_string()
        ),
        temp_exec(
            "repo_root".to_string(),
            "git rev-parse --show-toplevel".to_string(),
            "Error getting repo root".to_string()
        ),
        get_lookup,
        iife
    );

    let finb = fin.as_bytes();
    finb.to_vec()
}

