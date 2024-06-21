use crate::interpretable::InputType::Filename;
use crate::manager::prepare_render_outside;
use crate::manager::RenderManager;
use crate::ui::were_so_cool_logo;
use colored::*;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rand::Rng;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use weresocool_error::Error;

pub fn watch(
    filename: String,
    working_path: PathBuf,
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<(), Error> {
    were_so_cool_logo(Some("Watching"), Some(filename.clone()));

    let path = Path::new(&working_path).join(Path::new(&filename));

    std::thread::spawn(move || -> Result<(), Error> {
        loop {
            let (tx, rx) = channel();

            let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

            // should be ?
            watcher
                .watch(path.as_ref(), RecursiveMode::NonRecursive)
                .unwrap();

            if let Ok(_event) = rx.recv() {
                std::thread::sleep(std::time::Duration::from_millis(100));

                render(&filename, &working_path, &render_manager);
            }
        }
    });

    Ok(())
}

fn render(filename: &str, working_path: &Path, render_manager: &Arc<Mutex<RenderManager>>) {
    let render_voices =
        match prepare_render_outside(Filename(filename), Some(working_path.to_path_buf())) {
            Ok(result) => Some(result),
            Err(error) => {
                println!("{}", error);
                None
            }
        };

    if let Some(voices) = render_voices {
        render_manager.lock().unwrap().push_render(voices, false);
        let mut rng = rand::thread_rng();

        print!(
            "{} ",
            "* ".truecolor(rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>())
                .bold()
        );
        io::stdout().flush().unwrap();
    }
}
