//! The UI on `FakeHost`'s demo data: `cargo run -p aat-tui --example fake`.
//!
//! `AAT_ASCII=1` draws ASCII only; `NO_COLOR=1` draws no colour; `AAT_THEME=light` uses the deeper
//! shades. `e` opens sample files written to the system's temporary folder.

fn main() -> std::io::Result<()> {
    aat_tui::run_fake(aat_tui::TuiOptions::from_env())
}
