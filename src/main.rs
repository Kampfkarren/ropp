// TODO: Use clap instead so that CI doesn't need to stdin
use std::env;
use std::fs;
use std::io::{self, stdout, Write};
use std::process::{Command, ExitStatus};

mod publish;
mod structures;

const ROBLOSECURITY_HEADER: &str = "_|WARNING:-DO-NOT-SHARE-THIS.--";

#[derive(Debug)]
enum RunStepError {
    BadExit(ExitStatus),
    ExecuteError(io::Error),
}

fn run_steps(
    steps: &[structures::Step],
    relative_path: &str,
    tmp: &str,
) -> Result<(), RunStepError> {
    for step in steps {
        println!("step: {}", step.join(" "));
        let mut step_iter = step.iter();
        let status = Command::new(step_iter.next().unwrap()) // Step is guaranteed to have at least one parameter
            .env("ROPP_RBXMX", tmp)
            .current_dir(relative_path)
            .args(step_iter)
            .status()
            .map_err(RunStepError::ExecuteError)?;
        if !status.success() {
            return Err(RunStepError::BadExit(status));
        }
    }

    Ok(())
}

#[test]
fn test_run_steps() {
    macro_rules! test {
        ($test:expr, $steps:expr, $expected:pat => $then:block) => {
            match run_steps(
                &($steps
                    .iter()
                    .map(|steps| steps.iter().map(|s| s.to_string()).collect())
                    .collect::<Vec<Vec<String>>>()),
                ".",
                ".",
            ) {
                $expected => $then,
                _other => {
                    panic!("{} test failed! Got {:?} instead", $test, _other);
                }
            }
        };
    }

    test!(
        "BadExit",
        &[
            vec!["python", "./test/exit_42.py"],
        ],
        Err(RunStepError::BadExit(status)) => {
            assert_eq!(status.code(), Some(42))
        }
    );

    test!(
        "ExecuteError",
        &[
            vec!["thisprogramdefinitelydoesntexistalsodogeisfunny"],
        ],
        Err(RunStepError::ExecuteError(_)) => {}
    );

    test!(
        "OK",
        &[
            vec!["echo"],
        ],
        Ok(()) => {}
    );
}

fn main() {
    println!("ropp {}", env!("CARGO_PKG_VERSION"));

    let mut args = env::args();
    args.next();

    macro_rules! next_arg {
        ($problem:tt) => {
            match args.next() {
                Some(arg) => arg,
                None => {
                    eprintln!($problem);
                    eprintln!("Usage: ropp directory rbxmx build");
                    eprintln!("Read the documentation for more info");
                    return;
                }
            }
        };
    }

    let directory = next_arg!("No directory passed in");
    let rbxmx_path = next_arg!("No rbxmx file passed in");
    let build = next_arg!("No build passed in");

    let ropp_config: structures::Config = match serde_json::from_str(&match fs::read_to_string(
        format!("{}/ropp.json", directory),
    ) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Error trying to read ropp.json in directory: {}", error);
            return;
        }
    }) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Error trying to read ropp.json: {}", error);
            return;
        }
    };

    if let Err(error) = ropp_config.validate() {
        eprintln!("Error validating ropp.json: {}", error);
        return;
    }

    let mut rbxmx = match fs::read_to_string(&rbxmx_path) {
        Ok(rbxmx) => rbxmx,
        Err(error) => {
            eprintln!("Error trying to read rbxmx {}", error);
            return;
        }
    };

    let build_info = match ropp_config.build_info(&build) {
        Some(build_info) => build_info,
        None => {
            eprintln!("Couldn't find build {}", build);
            return;
        }
    };

    println!("directory: {}", directory);
    println!("rbxmx: {}", rbxmx_path);
    println!("build: {}", build);

    println!("creating temp file for rbxmx");

    let mut tempfile = match tempfile::NamedTempFile::new() {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error trying to create temp file: {}", error);
            return;
        }
    };

    if let Err(error) = write!(tempfile, "{}", rbxmx) {
        eprintln!("Error writing to temp file: {}", error);
        return;
    }

    let tmp_path = tempfile.path().to_string_lossy();
    println!("temp file: {}", tmp_path);

    macro_rules! run_steps {
        ($steps:ident) => {
            if let Err(error) = run_steps($steps, &directory, &tmp_path) {
                eprintln!("Error running step!");

                eprintln!(
                    "{}",
                    match error {
                        RunStepError::BadExit(status) => {
                            format!("Step returned a bad exit code: {}", status)
                        }

                        RunStepError::ExecuteError(error) => {
                            format!("Step failed to execute: {}", error)
                        }
                    }
                );

                return;
            }
        };
    }

    if let Some(pre) = &build_info.1.pre {
        println!("{} pre-steps to run", pre.len());
        run_steps!(pre);
        println!("finished running pre-steps");
        rbxmx = match fs::read_to_string(&tmp_path.clone().into_owned()) {
            Ok(rbxmx) => rbxmx,
            Err(error) => {
                eprintln!("Error reading temporary file again: {}", error);
                return;
            }
        };
    } else {
        println!("no pre-steps given");
    }

    println!("In order to publish your place, Ropp needs your .ROBLOSECURITY");
    println!(
        "You can find your .ROBLOSECURITY by reading your Cookie in an HTTP request to Roblox"
    );
    println!("Read the documentation for more details");
    println!("Because of the sensitivity of this cookie, it will not be shown in the console");

    let mut roblosecurity;

    loop {
        print!("ROBLOSECURITY: ");
        stdout().flush().expect("Couldn't flush stdout?");
        roblosecurity =
            rpassword::read_password().expect("Couldn't read ROBLOSECURITY for some reason?");
        println!();

        match roblosecurity.matches(ROBLOSECURITY_HEADER).count() {
            0 => {
                eprintln!("ROBLOSECURITY invalid, didn't start with the proper header");
                eprintln!("You probably copied and pasted it wrong");
            }

            1 => break,

            _ => {
                eprintln!("ROBLOSECURITY header repeated");
                eprintln!("You probably copied and pasted it too many times");
            }
        };
    }

    println!("publishing");

    if let Err(error) = publish::upload_place(build_info.0, rbxmx, &roblosecurity) {
        eprintln!("There was an error while publishing.");
        eprintln!("{}", error);
    } else {
        println!("published successfully!");
        if let Some(post) = &build_info.1.post {
            println!("{} post-steps to run", post.len());
            run_steps!(post);
            println!("finished running post-steps");
        }
    }
}
