# Reference host parser

The JavaScript files in `dist/` are the parser and its transitive dependencies
from `pi-automations` commit `f5cf5cd67cd441654692b4ddc72f23f6186e08cd`.
They are used only by tests to check the documented examples against a known
reference implementation. Its diagnostics identify that snapshot's Draft 0.2; the
tests normalize that version label when comparing them with Draft 0.1.0. Source-map
references have been removed. Update this snapshot deliberately when checking
examples against a newer reference host.
