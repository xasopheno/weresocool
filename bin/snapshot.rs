use weresocool::testing::{
    generate_test_table, read_test_table_from_json_file, show_difference,
    write_test_table_to_json_file,
};

fn main() {
    println!("\nHello Danny's WereSoCool Tests");
    let should_rehash = std::env::args().any(|x| x == "--rehash");

    let test_table = generate_test_table();

    if should_rehash {
        write_test_table_to_json_file(&test_table);
        println!("TestsSoRehashed");
    } else {
        let decoded = read_test_table_from_json_file();

        if test_table == decoded {
            println!("All Snapshot Tests Passed");
        } else {
            show_difference(decoded, test_table);
            panic!()
        }
    }
}
