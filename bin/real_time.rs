use failure::Fail;
use std::sync::{Arc, Mutex};
use weresocool::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    manager::RenderManager,
    portaudio::real_time_render_manager,
    ui::{get_args, no_file_name, were_so_cool_logo},
};
use weresocool_error::Error;
use weresocool_instrument::renderable::{nf_to_vec_renderable, renderables_to_render_voices};

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
}

fn run() -> Result<(), Error> {
    were_so_cool_logo();
    println!("       )))***=== REAL<COOL>TIME *buffered ===***(((  \n ");

    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let (nf, basis, table) =
        match Filename(filename.unwrap()).make(RenderType::NfBasisAndTable, None)? {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => panic!("Error. Unable to generate NormalForm"),
        };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;
    let render_voices = renderables_to_render_voices(renderables);

    let render_manager = Arc::new(Mutex::new(RenderManager::init(render_voices)));

    let mut stream = real_time_render_manager(Arc::clone(&render_manager)).unwrap();
    stream.start().unwrap();
    while let true = stream.is_active().unwrap() {}
    stream.stop().unwrap();

    Ok(())
}
