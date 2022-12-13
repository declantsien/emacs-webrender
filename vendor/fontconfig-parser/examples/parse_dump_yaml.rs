#[cfg(feature = "serde")]
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example parse_dump_yaml -- <conf file path>");
        return;
    }

    let parts =
        fontconfig_parser::parse_config_parts(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();

    serde_yaml::to_writer(std::io::stdout(), &parts).unwrap();
}

#[cfg(not(feature = "serde"))]
fn main() {}
