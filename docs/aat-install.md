# Installing aat

`aat` is one binary with no runtime to install. You build it from source with Rust 1.85 or later
([rustup.rs](https://rustup.rs)), on Windows, macOS or Linux.

```sh
git clone https://github.com/lucas-stanford/agent-automations
cd agent-automations/aat
cargo install --locked --path crates/aat
```

Cargo puts `aat` in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on Windows), which rustup adds to
your PATH. Check it with `aat --version`.

Other software, including [Adaptive API Toolkit](https://gburgyan.github.io/aat/install/), also
uses the `aat` command. Check `command -v aat` on macOS or Linux, or `Get-Command aat` in
PowerShell. If another executable takes precedence, run this host by its full path rather than
overriding the other tool.

To upgrade, pull and run the same `cargo install` again. On Windows it cannot replace `aat.exe`
while a run is using it; if it fails for that reason, wait for the run to end and try again.

## Uninstall

1. Stop the watcher for this state folder, if one is registered: `aat watcher stop`.
2. Remove the binary: `cargo uninstall aat`.
3. If you also want aat's state gone (consent, run records, config), delete its state folder:
   `AAT_HOME` if you set it, else the per-OS folder in the
   [README](aat.html#where-state-lives).

Uninstalling leaves your automations alone, both in `~/.agents/automations/` and in each
repository's `.agents/automations/`.
