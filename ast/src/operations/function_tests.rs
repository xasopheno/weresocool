
fn test_function() -> Result<(), Error> {
    let render_return = Filename("./mocks/simple_fun.socool").make(RenderType::NfBasisAndTable)?;
    let (nf, basis, table) = match render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };

