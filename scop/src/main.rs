use scop::{Defs, ScopError};
pub type Term = i32;

fn main() -> Result<(), ScopError> {
    println!("Scop!");
    let mut defs: Defs<Term> = Defs::new();

    defs.insert("global", "1", 1);
    defs.insert("global", "2", 2);

    let new_scope = defs.create_uuid_scope();
    defs.insert(&new_scope, "3", 3);

    let result = defs.get("3");
    dbg!(&defs);
    dbg!(result);

    Ok(())
}
