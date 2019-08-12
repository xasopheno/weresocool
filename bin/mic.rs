use std::io::{stdout, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::raw::IntoRawMode;
use weresocool::{
    examples::documentation,
    generation::{filename_to_render, RenderReturn, RenderType},
    portaudio::duplex_setup,
    ui::{get_args, no_file_name, were_so_cool_logo},
};

use portaudio as pa;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn setup_termion(x: Arc<Mutex<String>>) {
	let mut stdin = async_stdin().bytes();

    thread::spawn(move || 
	loop {
        let b = stdin.next();
        match b {
	    Some(Ok(b'r')) => {
                *x.lock().unwrap() = "recording".to_string();
		//break;
	    },
	    Some(Ok(b'q')) => {
                *x.lock().unwrap() = "quit".to_string();
	    }
            _ => {},
        };
    thread::sleep(std::time::Duration::from_millis(100));
    }

    );
}

fn run() -> Result<(), pa::Error> {
    were_so_cool_logo();
    println!("{}", "       )))***=== MICROPHONE ===***(((  \n ");

    let args = get_args();

    if args.is_present("doc") {
        documentation();
    }

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let normal_form = match filename_to_render(filename.unwrap(), RenderType::NfBasisAndTable) {
        RenderReturn::NfAndBasis(nf, _, _) => nf,
        _ => panic!("Error. Unable to generate NormalForm"),
    };

    println!("\nGenerating Composition ");
    let mut x: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
    setup_termion(Arc::clone(&x));

    let mut duplex_stream = duplex_setup(Arc::clone(&x), normal_form.operations)?;
    duplex_stream.start()?;
    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}
