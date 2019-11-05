use error::Error;
use weresocool::{
    //error::Error,
    examples::documentation,
    generation::{filename_to_render, generate_waveforms, RenderReturn, RenderType},
    portaudio::output_setup,
    ui::{banner, get_args, no_file_name, were_so_cool_logo},
    write::write_composition_to_wav,
};

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let render_return = filename_to_render(filename.unwrap(), RenderType::NfBasisAndTable)?;
    let (nf, basis, table) = match render_return {
        RenderReturn::NfAndBasis(nf, basis, _) => (nf, basis, _),
        _ => panic!("huh"),
    };
    let vec_wav = generate_waveforms(&basis, nf.operations, true);
    for (i, w) in vec_wav.iter().enumerate() {
        let f = format!("{}_{}.wav", &filename.clone().unwrap(), i);
        let f = f.split("/").collect::<Vec<&str>>();
        let f = f[f.len() - 1];
        dbg!(&f);

        write_composition_to_wav(w.clone(), &f);
    }
    //let mut result = sum_all_waveforms(vec_wav);

    Ok(())
}
