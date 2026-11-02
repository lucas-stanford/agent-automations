//! `aat`: with no arguments, the terminal UI; with a subcommand, the scriptable CLI.

fn main() {
    std::process::exit(aat::cli::main());
}
