use crate::Error;
use colored::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use weresocool::interpretable::InputType::Filename;
use weresocool::manager::prepare_render_outside;
use weresocool::manager::RenderManager;

pub fn watch(
    filename: String,
    working_path: PathBuf,
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<(), Error> {
    let mut first_iteration = true;
    std::thread::spawn(move || -> Result<(), Error> {
        loop {
            if first_iteration {
                render(&filename, &working_path, &render_manager);
                first_iteration = false;
            }

            let (tx, rx) = channel();

            let mut watcher = RecommendedWatcher::new(tx).unwrap();

            let path = Path::new(&working_path).join(Path::new(&filename));

            watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
            if let Ok(_event) = rx.recv() {
                std::thread::sleep(std::time::Duration::from_millis(100));

                render(&filename, &working_path, &render_manager);
            }
        }
    });

    Ok(())
}

fn render(filename: &str, working_path: &PathBuf, render_manager: &Arc<Mutex<RenderManager>>) {
    let render_voices =
        match prepare_render_outside(Filename(&filename), Some(working_path.clone())) {
            Ok(result) => Some(result),
            Err(error) => {
                println!("{}", error);
                None
            }
        };

    if let Some(voices) = render_voices {
        render_manager.lock().unwrap().push_render(voices);
        print!("{}", ". ".magenta().bold());
        io::stdout().flush().unwrap();
    }
}
