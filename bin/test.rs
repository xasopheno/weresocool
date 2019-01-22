extern crate num_rational;
extern crate weresocool;
use weresocool::testing::{
    generate_test_table, read_test_table_from_json_file, show_difference,
    write_test_table_to_json_file,
};

fn main() {
    println!("\nHello Danny's WereSoCool Scratch Pad");
    //  1) Generate Hashes --bin test --re-hash
    //  2) Run All Tests --bin test --all
    //  3) Run Single Test --bin test --file ./songs/test/pan_test.socool
    let test_table = generate_test_table();

    if false {
        write_test_table_to_json_file(&test_table);
    } else {
        let decoded = read_test_table_from_json_file();

        if test_table == decoded {
            println!("All Tests Passed");
        } else {
            show_difference(decoded, test_table);
            println!("Error above");
        }
    }
}
