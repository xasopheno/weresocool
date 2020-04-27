use weresocool::{
    generation::{generate_waveforms, RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
    renderable::nf_to_vec_renderable,
    ui::{get_args, no_file_name, were_so_cool_logo},
    write::write_composition_to_wav,
};
use weresocool_error::Error;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let render_return = Filename(filename.unwrap()).make(RenderType::NfBasisAndTable)?;
    let (nf, basis, table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };
    let renderables = nf_to_vec_renderable(&nf, &table, &basis)?;
    let vec_wav = generate_waveforms(renderables, true);
    for (i, w) in vec_wav.iter().enumerate() {
        let f = format!("{}_{}.wav", &filename.clone().unwrap(), i);
        let f = f.split('/').collect::<Vec<&str>>();
        let f = f[f.len() - 1];
        println!("{}", &f);

        write_composition_to_wav(w.clone(), &f, false, false);
    }

    Ok(())
}
