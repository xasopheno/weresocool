//#![allow(dead_code, unused_imports, unused_variables)]
use failure::Fail;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use weresocool::{
    generation::parsed_to_render::{RenderReturn, RenderType},
    interpretable::{InputType, Interpretable},
    manager::RenderManager,
    portaudio::real_time_render_manager,
    ui::were_so_cool_logo,
};
use weresocool_error::Error;
use weresocool_instrument::renderable::{
    nf_to_vec_renderable, renderables_to_render_voices, RenderVoice,
};

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

pub fn prepare_render(input: InputType<'_>) -> Result<Vec<RenderVoice>, Error> {
    let (nf, basis, table) = match input.make(RenderType::NfBasisAndTable)? {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;

    let render_voices = renderables_to_render_voices(renderables);

    Ok(render_voices)
}

fn run() -> Result<(), Error> {
    were_so_cool_logo();
    println!("       )))***=== REAL<COOL>TIME *buffered ===***(((  \n ");

    let filename1 = "songs/dance/skip.socool";
    let filename2 = "songs/dance/candle.socool";

    let render_manager = Arc::new(Mutex::new(RenderManager::init_silent()));
    let render_manager_clone = render_manager.clone();

    let (send, recv) = channel();
    println!("Start...");
    thread::Builder::new()
        .name("Sender".to_string())
        .spawn(move || {
            thread::sleep(Duration::from_secs(1));
            for _ in 0..4 {
                send.send(InputType::Filename(&filename1)).unwrap();
                thread::sleep(Duration::from_secs(4));
                send.send(InputType::Filename(&filename2)).unwrap();
                thread::sleep(Duration::from_secs(4));
            }
        })?;

    thread::Builder::new()
        .name("Receiver".to_string())
        .spawn(move || loop {
            if let Ok(v) = recv.try_recv() {
                println!("language received");
                let render = prepare_render(v);

                match render {
                    Ok(r) => {
                        render_manager_clone.lock().unwrap().push_render(r);
                        println!("Render Success")
                    }
                    _ => panic!("Render Failure"),
                }
            };
        })?;

    let mut stream = real_time_render_manager(Arc::clone(&render_manager))?;
    stream.start()?;

    while let true = stream.is_active()? {}

    stream.stop()?;

    Ok(())
}
