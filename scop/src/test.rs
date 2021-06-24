#[allow(unused_imports)]
use crate::{Defs, ScopError};

#[test]
fn it_can_insert_and_find_in_global_scope() -> Result<(), ScopError> {
    let mut defs: Defs<i32> = Defs::new();

    defs.insert("global", "1", 1);
    defs.insert("global", "2", 2);

    let found = defs.get("1");
    assert_eq!(found, Some(&1));
    let not_found = defs.get("3");
    assert_eq!(not_found, None);
    Ok(())
}

#[test]
fn it_can_insert_and_find_with_uuid_scope_name() -> Result<(), ScopError> {
    let mut defs: Defs<i32> = Defs::new();

    defs.insert("global", "1", 1);
    defs.insert("global", "2", 2);

    let new_scope = defs.create_uuid_scope();
    defs.insert(&new_scope, "3", 3);

    let found = defs.get("3");
    assert_eq!(found, Some(&3));
    Ok(())
}

#[test]
fn it_finds_value_in_innermost_scope() -> Result<(), ScopError> {
    let mut defs: Defs<i32> = Defs::new();

    defs.insert("global", "1", 1);
    defs.insert("global", "2", 2);

    let new_scope = defs.create_uuid_scope();
    defs.insert(&new_scope, "1", 10);

    let found = defs.get("1");
    assert_eq!(found, Some(&10));
    Ok(())
}
