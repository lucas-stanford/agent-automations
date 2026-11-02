# aat

`aat` is a host for Agent Automations. It finds `AUTOMATION.md` files, runs the ones you arm, and
sends each run through the agent CLI you pick: pi, GitHub Copilot CLI, Claude Code, Codex CLI,
Gemini CLI or OpenCode. Runs start on a schedule, when a watch script finds work, or by hand. The
terminal UI shows what needs you.

## Quickstart

1. Install `aat` ([docs/INSTALL.md](docs/INSTALL.md)) and check it with `aat --version`.

2. Pick the agent CLI that runs your automations. `aat agent list` shows the ones installed here.

   ```sh
   aat agent list
   aat agent default copilot
   ```

3. Write an automation, or start from `watch-inbox` in
   [tests/conformance/valid/watch-inbox](tests/conformance/valid/watch-inbox). It summarises each
   text file that lands in its `inbox` folder, and its two scripts need Node.js.

   ```markdown
   ---
   name: watch-inbox
   description: When new text files appear in this automation's inbox folder, summarise each one with its action items.
   timeout: 15m
   on:
     watch:
       script: scripts/watch.mjs
       every: 5m
     manual: true
   hooks:
     pre:
       - script: scripts/read_inbox.mjs
   ---

   # Summarise the inbox

   The input block below holds every file waiting in `inbox/`, …
   ```

   Copy the folder, with its `scripts`, into your user automations, and give it a file to read.
   From the `aat/` directory of a clone, on macOS or Linux:

   ```sh
   mkdir -p ~/.agents/automations
   cp -r tests/conformance/valid/watch-inbox ~/.agents/automations/
   mkdir -p ~/.agents/automations/watch-inbox/inbox
   echo "Sam sends the budget by Friday." > ~/.agents/automations/watch-inbox/inbox/notes.txt
   ```

   On Windows, in PowerShell:

   ```powershell
   New-Item -ItemType Directory -Force $HOME\.agents\automations
   Copy-Item -Recurse tests\conformance\valid\watch-inbox $HOME\.agents\automations\
   New-Item -ItemType Directory -Force $HOME\.agents\automations\watch-inbox\inbox
   Set-Content $HOME\.agents\automations\watch-inbox\inbox\notes.txt "Sam sends the budget by Friday."
   ```

4. Check it. `aat check` shows its state and any problems, then the consent preview: the exact
   agent command, the model, the hooks and the files consent is pinned to.

   ```sh
   aat check watch-inbox
   ```

5. Arm it. `aat arm` shows the consent preview again and asks before arming.

   ```sh
   aat arm watch-inbox
   ```

6. Run it by hand, then read its runs.

   ```sh
   aat run watch-inbox
   aat log watch-inbox
   ```

7. Start the watcher, which fires schedules and runs watch checks. It is an OS job that ticks once
   a minute, with or without a terminal open.

   ```sh
   aat watcher start
   aat watcher status
   ```

When you edit `AUTOMATION.md` or one of its scripts, the automation turns `off` until you arm it
again. A failed run shows a desktop notification; set `notify-on-failed = false` in `config.toml`,
or `AAT_NO_NOTIFY=1`, to turn that off. [docs/CLI.md](docs/CLI.md) lists every command.

## Reporting from inside a run

The body, its hooks and anything they start can report progress with `aat step`, `aat next`,
`aat say`, `aat note`, `aat message` and `aat status`. aat sets `AAT_RUN` for them, so these commands
find the run from any folder, and the terminal UI and `aat log` show what they record. A body that
opens a pull request might end with:

```text
aat note "Opened the review" --link "PR=https://github.com/org/app/pull/42"
aat message "Reviewed 3 files; one question for you in the PR."
```

[docs/CLI.md](docs/CLI.md#inside-a-run) lists their flags and exit codes.

## The terminal UI

`aat` with no command opens it; `aat tui --fake` opens it on demo data.

The list page has five sections: Needs you, Running now, Coming up, Watching, and Off and
disabled. `enter` opens an automation page with what it does, its last run and every action. A
run page shows one run, with its steps or flowchart, live output, notes and message.

| Key | Does |
|---|---|
| `↑` `↓`, `j` `k` | Move. `PgUp` `PgDn` `Home` `End` move or scroll further. |
| `enter` | Open the automation page. |
| `r` | Run it now. |
| `x` | Stop its run. It asks first. |
| `s`, `l` | Show its run; its runs. Both open the run page. |
| `space` | Arm an `off` automation after the consent preview, enable a `disabled` one, disable an armed user automation, or disarm an armed repository automation. |
| `p` | Pause or resume its watch. |
| `e` | Edit `AUTOMATION.md` in `$VISUAL` or `$EDITOR`. |
| `a` | Choose its agent CLI. |
| `c` | The check page: every technical detail. |
| `g` | On a run page: the flowchart, or back to the steps. |
| `o` | On a run page: open a link the run published. |
| `y` | Yes, when asked. In the consent preview, only `y` arms. |
| `R` | Read everything again. |
| `?` | Every key. |
| `esc`, `q` | `esc` goes back; `q` closes. |

`AAT_THEME=light` or `dark` picks colours when the terminal does not say; `AAT_ASCII=1` draws with
ASCII only; `NO_COLOR` turns colour off.

## Agent CLIs

| CLI | Name in aat | Invocation (prompt on stdin) | Resume | Tool approval flag |
|---|---|---|---|---|
| pi | `pi` | `pi -p --mode json` | `--session <id>` | none: pi never asks |
| GitHub Copilot CLI | `copilot` | `copilot --output-format json` | `--resume=<id>` | `--allow-all-tools` |
| Claude Code | `claude` | `claude -p --output-format stream-json --verbose` | `--resume <id>` | `--dangerously-skip-permissions` |
| Codex CLI | `codex` | `codex exec --json --skip-git-repo-check -` | `exec resume <id> -` | `--sandbox workspace-write` |
| Gemini CLI | `gemini` | `gemini --output-format stream-json` | `--resume <id>` | `--approval-mode yolo` |
| OpenCode | `opencode` | `opencode run --format json` | `--session <id>` | `--auto` |

aat adds the tool approval flag only when you set `allow-all-tools = true` under `[agents.<cli>]` in
`config.toml`. Prompt hooks continue the body's session through the resume flag. aat refuses a
model the CLI cannot serve and never picks another. [docs/AGENTS.md](docs/AGENTS.md) has the model
flags, the versions checked and each CLI's limits.

## aat and pi-automations

pi-automations (`pa`) is another host for the same format, built into the pi coding agent. `aat`
keeps its own consent and state, so arming an automation in one host does not arm it in the other.

Both read `~/.agents/automations/` and `.agents/automations/`, so you can arm the same automation
in both, and then both run it. `aat` reads pi-automations' `trust.json` without writing to it
(`$PI_AGENT_DIR/pi-automations/trust.json`, by default `~/.pi/agent/pi-automations/trust.json`) and
warns when an automation is armed there too:

```text
  watch-inbox       armed     watch 5m      check now   copilot
    ! also armed in pa — it would run twice; pa disarm watch-inbox
```

`aat list`, `aat check`, the consent preview and the UI all show this warning. Disarm the automation
in one host.

`aat` reads only the folders the specification names, so it skips pi-automations'
`~/.pi/agent/automations/` and `.pi/automations/`. Its settings live in `config.toml`
([docs/CONFIG.md](docs/CONFIG.md)) instead of pi-automations' `config.json`, and some defaults
differ.

## Where state lives

aat keeps consent (`trust.json`), `state.json`, `config.toml`, run records (`runs/`), locks,
progress and agent output (`sessions/`) in one state folder outside any repository. `AAT_HOME` sets
it; otherwise:

| OS | State folder |
|---|---|
| Windows | `%LOCALAPPDATA%\agent-automations` |
| macOS | `~/Library/Application Support/agent-automations` |
| Linux | `$XDG_STATE_HOME/agent-automations`, else `~/.local/state/agent-automations` |

Automations live in `~/.agents/automations/` (user scope) and in a repository's
`.agents/automations/` (repository scope). `aat` finds the home folder from `USERPROFILE` on Windows
and `HOME` elsewhere.

## Docs

- [The specification](../site/content/specification.md), and the format's other pages in
  [site/content](../site/content): validation messages, watch scripts, flowcharts, and adding
  support to a host.
- [docs/INSTALL.md](docs/INSTALL.md): installing `aat`.
- [docs/CLI.md](docs/CLI.md): every command, flag, exit code and JSON output.
- [docs/CONFIG.md](docs/CONFIG.md): every `config.toml` key.
- [docs/AGENTS.md](docs/AGENTS.md): how aat runs each agent CLI.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how `aat` is built.

## License

MIT
