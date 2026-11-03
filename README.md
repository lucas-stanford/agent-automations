# Agent Automations

The Agent Automations Draft 0.2.0 specification describes a portable format for
running AI agents on a schedule, when a watch script finds work, or on demand.
Read the specification and guides at <https://lucas-stanford.github.io/agent-automations/>.
The host-neutral agent skill is in [skills/agent-automations](skills/agent-automations/SKILL.md).

## Hosts

[fm](https://github.com/lucas-stanford/fm) is the reference host: a command line and terminal UI
that runs automations through pi, GitHub Copilot CLI, Claude Code, Codex CLI, Gemini CLI or
OpenCode. It implements every page of the specification, and `fm validate <AUTOMATION.md>`
checks a file as the specification says. [pi-automations](https://github.com/lucas-stanford/pi-automations)
brings fm into the pi coding agent.

## Build and check

With Node.js 22.19 or newer, and [fm](https://github.com/lucas-stanford/fm) on your `PATH` (or
`FM_BIN` set to it):

```sh
npm ci
npm run build
npm test
```

Edit the Markdown in `site/content/` and run the build to regenerate `docs/`. Open
`docs/index.html` directly to browse the site. The tests check links, the committed build output,
and every example against fm: each is accepted or refused exactly as the page says.

## License

[MIT](LICENSE)
