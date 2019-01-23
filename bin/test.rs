extern crate num_rational;
extern crate weresocool;
use weresocool::{
    testing::{
        generate_test_table, read_test_table_from_json_file, show_difference,
        write_test_table_to_json_file,
    },
    ui::get_test_args,
};

fn main() {
    println!("\nHello Danny's WereSoCool Tests");
    let test_table = generate_test_table();

    let args = get_test_args();

    if args.is_present("rehash") {
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
