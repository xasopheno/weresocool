use scop::{Defs, ScopError, Term};

fn main() -> Result<(), ScopError> {
    println!("Scop!");
    let mut defs: Defs<Term> = Defs::new();

    defs.insert("global", "1", 1)?;
    defs.insert("global", "2", 2)?;

    let new_scope = defs.create_uuid_scope();
    defs.insert(&new_scope, "3", 3)?;

    let result = defs.substitute("id");
    dbg!(defs);
    dbg!(result);

    Ok(())
}
