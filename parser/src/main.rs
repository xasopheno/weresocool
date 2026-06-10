use colored::*;
use std::env;
use std::process::ExitCode;
use weresocool_parser::parser::filename_to_vec_string;
use weresocool_parser::*;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let filename;
    if args.len() == 2 {
        filename = &args[1];
    } else {
        eprintln!("\n{}\n", "Forgot to pass in a filename.".red().bold());
        eprintln!("{}", "Example:".cyan());
        eprintln!("{}\n", "./weresocool song.socool".cyan().italic());
        return ExitCode::FAILURE;
    }

    let vec_string = match filename_to_vec_string(filename) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("\n{}\n", e.to_string().red());
            return ExitCode::FAILURE;
        }
    };

    match parse_file(vec_string, None, None, Some(filename.to_string())) {
        Ok(parsed) => {
            for (key, _val) in parsed.defs.ops.iter() {
                println!("{}", key);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("\n{}\n", e.to_string().red());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod test;
