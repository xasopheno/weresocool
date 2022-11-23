use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use weresocool_core::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    manager::{RenderManager, VisEvent},
    portaudio::real_time_render_manager,
    ui::{get_args, no_file_name, were_so_cool_logo},
};
use weresocool_error::Error;
use weresocool_instrument::renderable::{nf_to_vec_renderable, renderables_to_render_voices};

fn main() -> Result<(), Error> {
    were_so_cool_logo(None, None);
    println!("       )))***=== REAL<COOL>TIME *buffered ===***(((  \n ");

    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let (nf, basis, mut table) =
        match Filename(filename.unwrap()).make(RenderType::NfBasisAndTable, None)? {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => panic!("Error. Unable to generate NormalForm"),
        };
    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis)?;
    let render_voices = renderables_to_render_voices(renderables);

    let (tx, rx): (Sender<VisEvent>, Receiver<VisEvent>) = crossbeam_channel::unbounded();

    let render_manager = Arc::new(Mutex::new(RenderManager::init(
        render_voices,
        Some(tx),
        None,
        false,
    )));

    let mut stream = real_time_render_manager(Arc::clone(&render_manager))?;
    stream.start()?;
    while let true = stream.is_active()? {
        if let Ok(x) = rx.recv() {
            dbg!(x);
        }
    }
    stream.stop()?;

    Ok(())
}
