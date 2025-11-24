use std::sync::{Arc, Mutex};
use weresocool_core::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    manager::RenderManager,
    portaudio::create_portaudio_stream,
};
use weresocool_error::Error;
use weresocool_instrument::renderable::{nf_to_vec_renderable, renderables_to_render_voices};

fn main() -> Result<(), Error> {
    let filename = "test.socool";

    let (nf, basis, mut table) = match Filename(filename).make(RenderType::NfBasisAndTable, None)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };
    let renderables = nf_to_vec_renderable(&nf, &mut table, &basis)?;
    let render_voices = renderables_to_render_voices(renderables);

    let render_manager = Arc::new(Mutex::new(RenderManager::init(
        None,
        None,
        false,
        None,
    )));

    // Subscribe to render events
    let rx = render_manager.lock().unwrap().events.render.subscribe();

    render_manager.lock().unwrap().push_render(render_voices, false);

    let mut stream = create_portaudio_stream(Arc::clone(&render_manager))?;
    stream.start()?;
    while let true = stream.is_active()? {
        if let Ok(x) = rx.recv() {
            dbg!(x);
        }
    }
    stream.stop()?;

    Ok(())
}
