use crate::{
    generation::{RenderReturn, RenderType},
    interpretable::{InputType::Filename, Interpretable},
};
use weresocool_error::Error;

/// Function for testing that two .socool files are equivilant.
/// ```
///
/// # use weresocool::testing::expect::{expect_eq};
/// fn test_expect_eq() {
///     let _ = expect_eq(
///        "mocks/input.socool",
///        "mocks/expected.socool",
///     );
/// }
/// ```
pub fn expect_eq(input: &str, expected: &str) -> Result<(), Error> {
    let input_render_return = Filename(input).make(RenderType::NfBasisAndTable)?;
    let expected_render_return = Filename(expected).make(RenderType::NfBasisAndTable)?;

    let (nf_i, _basis_i, _table_i) = match input_render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };
    let (nf_e, _basis_e, _table_e) = match expected_render_return {
        RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
        _ => panic!("huh"),
    };

    assert_eq!(nf_i, nf_e);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expect_eq() {
        let _should_match = expect_eq(
            "src/testing/mocks/simple.socool",
            "src/testing/mocks/simple.socool",
        );
    }

    #[test]
    #[should_panic]
    fn test_expect_fail() {
        let _should_not_match = expect_eq("./mocks/simple.socool", "./mocks/simple.socool");
    }
}
