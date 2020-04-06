fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            rlox::run_prompt();
        },
        2 => {
            rlox::run_file(args.get(1).unwrap());
        },
        _ => {
            println!("Usage: rlox [script]");
        }
    }
}
