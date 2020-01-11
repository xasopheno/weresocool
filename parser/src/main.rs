use colored::*;
use socool_parser::parser::filename_to_vec_string;
use socool_parser::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename;
    if args.len() == 2 {
        filename = &args[1];
    } else {
        println!("\n{}\n", "Forgot to pass in a filename.".red().bold());
        println!("{}", "Example:".cyan());
        println!("{}\n", "./weresocool song.socool".cyan().italic());
        panic!("Wrong number of arguments.")
    }

    let vec_string = filename_to_vec_string(filename);
    let parsed = parse_file(vec_string, None);

    for (key, _val) in parsed.table.iter() {
        println!("\n Name: {:?}", key);
    }
}

#[cfg(test)]
mod test;
