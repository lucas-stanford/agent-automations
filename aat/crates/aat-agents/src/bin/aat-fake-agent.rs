//! Test-only stand-in for an agent CLI. See `aat_agents::fake` for the options.

fn main() {
    std::process::exit(aat_agents::fake::main());
}
