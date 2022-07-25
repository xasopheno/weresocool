use crate::Error;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use weresocool::core::interpretable::InputType::Filename;
use weresocool::core::manager::prepare_render_outside;
use weresocool::core::manager::RenderManager;

pub fn watch(
    filename: String,
    working_path: PathBuf,
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<(), Error> {
    std::thread::spawn(move || -> Result<(), Error> {
        loop {
            let (tx, rx) = channel();

            let mut watcher = RecommendedWatcher::new(tx).unwrap();

            let path = Path::new(&working_path).join(Path::new(&filename));

            watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
            if let Ok(_event) = rx.recv() {
                std::thread::sleep(std::time::Duration::from_millis(100));

                // println!("{:?}", event);
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
                }
            }
        }
    });
    Ok(())
}
