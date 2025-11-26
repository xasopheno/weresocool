use crate::Error;
use clap::ArgMatches;
use std::fs;
use std::io::{self, Read, Write};
use weresocool_formatter::{format_source, FormatConfig};

pub fn fmt(matches: &ArgMatches) -> Result<(), Error> {
    let config = FormatConfig::default();

    let (source, filename) = if matches.get_flag("stdin") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        (buffer, None)
    } else {
        let filename = matches.get_one::<String>("filename").unwrap();
        let source = fs::read_to_string(filename)?;
        (source, Some(filename.clone()))
    };

    let formatted = format_source(&source, &config)
        .map_err(|e| Error::Message(e.to_string()))?;

    if matches.get_flag("check") {
        // Check mode: exit 1 if not formatted
        if source != formatted {
            if let Some(f) = filename {
                eprintln!("Would reformat: {}", f);
            }
            return Err(Error::Message("File not formatted".to_string()));
        }
        // File is already formatted
        Ok(())
    } else if matches.get_flag("inplace") {
        // In-place mode: write back to file
        if let Some(f) = filename {
            fs::write(&f, &formatted)?;
            println!("Formatted: {}", f);
        } else {
            return Err(Error::Message("Cannot use --inplace with --stdin".to_string()));
        }
        Ok(())
    } else {
        // Default: print to stdout
        io::stdout().write_all(formatted.as_bytes())?;
        Ok(())
    }
}
