<div align="center">

# SkillNexus

**English** | [**简体中文**](README.md)

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-1.97-orange?style=flat-square)
![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square)
![TypeScript](https://img.shields.io/badge/TypeScript-blue?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)

**One skill library, connected to all your agents.**

A cross-platform desktop tool that manages the skill libraries of AI coding agents (Claude Code, Codex, Gemini CLI, DeepSeek Harness, Cursor, Cline …). A single `~/.agents/skills` directory acts as the single source of truth, mapped into each agent's skill directory via junctions / symlinks. Agents that natively read `~/.agents/skills` are connected directly, with zero configuration.

</div>

---

## ⬇️ Download

> Just want to use it? Grab the latest build for Windows — no compilation needed.

| File | Link |
|------|------|
| 📦 Windows installer | [SkillNexus_0.1.0_x64-setup.exe](https://github.com/Halface525/skill-nexus/releases/latest/download/SkillNexus_0.1.0_x64-setup.exe) |
| 🟢 Windows portable | [SkillNexus.exe](https://github.com/Halface525/skill-nexus/releases/latest/download/SkillNexus.exe) |

Other platforms or older versions: see [Releases](https://github.com/Halface525/skill-nexus/releases/latest).

> ~9MB installer, double-click to install on Win10/11. No runtime required.

---

## ✨ Features

- **Unified skill library** — all skills live in one directory; manage once, apply everywhere
- **One-click sync** — automatically creates links for every installed "junction" agent, so skills are ready instantly
- **Native direct-read support** — Codex, Cursor, DeepSeek Harness, Trae and others read `~/.agents/skills` natively
- **Agent detection** — scans which agents are installed on your machine and reports each skill directory's sync status
- **Install skills** — pick any folder containing a `SKILL.md`; it is copied into the library and linked automatically
- **SKILL.md rendering** — full Markdown rendering (headings / code blocks / tables / quotes) in the detail panel
- **Brand badges** — each agent shown as an official logo / brand-color badge with a corner status dot
- **Light / Dark / System** themes
- **Simplified Chinese / English** UI
- **Configurable library location** — defaults to `~/.agents/skills`, changeable anytime
- **Extensible agent list** — add or remove agents via a config file, no code changes

## 🗂️ Architecture

All agents' skills converge into one **unified library**, connected in two ways:

| Type | How it works | Example agents |
|------|--------------|----------------|
| **direct** | Agent natively reads `~/.agents/skills`, no link needed | DeepSeek Harness, Codex, Cursor, Trae, GitHub Copilot |
| **junction** | Agent has a fixed skill directory, linked to the library via **junction** (Windows) / **symlink** (macOS/Linux) | Claude, Cline, Gemini CLI, Qwen Code, Kiro |

```
                ┌─────────────────────┐
                │   ~/.agents/skills   │  ← Unified library (source of truth)
                └──────────┬──────────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
   junction/symlink        │          direct read
          │                │                │
   ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐
   │ ~/.claude/   │  │ ~/.gemini/  │  │ Codex /     │
   │  skills      │  │  skills     │  │ Cursor / …  │
   └─────────────┘  └─────────────┘  └─────────────┘
```

## 📸 Screenshots

![Main UI](docs/screenshot.png)

## 🖥️ Tech Stack

- **Desktop framework**: [Tauri 2](https://tauri.app/) (Rust core + system WebView)
- **Backend**: Rust — directory scanning, junction/symlink management, SKILL.md parsing
- **Frontend**: [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vite.dev/)
- **Icons**: official brand icons from simple-icons + inline SVG
- **Markdown rendering**: [react-markdown](https://github.com/remarkjs/react-markdown)

## 📦 Requirements

| Dependency | Version | Note |
|------------|---------|------|
| [Node.js](https://nodejs.org/) | ≥ 18 | Frontend build |
| [Rust](https://www.rust-lang.org/) | ≥ 1.77 | Backend compilation |

Tauri platform prerequisites (per OS):

- **Windows**: Microsoft C++ Build Tools (with the C++ workload), [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (built-in on Win11)
- **macOS**: Xcode Command Line Tools
- **Linux**: `webkit2gtk-4.1`, `libappindicator`, etc. See [Tauri prerequisites](https://tauri.app/start/prerequisites/)

## 🚀 Getting Started

```bash
# 1. Clone the repository
git clone https://github.com/Halface525/skill-nexus.git
cd skill-nexus

# 2. Install dependencies
npm install

# 3. Run in development mode (a desktop window opens automatically)
npm run tauri dev
```

## 📦 Build

```bash
npm run tauri build
```

Artifacts are produced in `src-tauri/target/release/`:

- `bundle/nsis/*.exe` — installer (recommended for sharing)
- `SkillNexus.exe` — standalone portable binary

> The built exe has no Rust / Node dependencies — double-click and run (WebView2 is built into Win10/11).

## ⚙️ Configuration

### Agent list — `~/.agents/agents.json`

Generated on first run; edit it to add or remove any agent without touching code:

```json
{
  "agents": [
    {
      "name": "Claude",
      "kind": "junction",
      "dir": ".claude/skills",
      "binary": "claude",
      "homeDir": ".claude"
    },
    {
      "name": "Codex",
      "kind": "direct",
      "binary": "codex",
      "homeDir": ".codex"
    }
  ]
}
```

| Field | Description |
|-------|-------------|
| `name` | Display name |
| `kind` | `direct` reads the unified library / `junction` creates a link |
| `dir` | Required for junction: skill directory (relative to home), e.g. `.claude/skills` |
| `binary` | Detection: executable name (optional) |
| `homeDir` | Detection: considered installed if this directory exists under home (optional) |
| `appdata` | Detection: considered installed if these directories exist under `%APPDATA%`, e.g. `["Trae CN"]` (optional) |

### Library location — `~/.agents/settings.json`

```json
{
  "unifiedLibrary": "D:/MySkills"
}
```

The default location is `~/.agents/skills`. You can also change or reset it anytime from the app: **Settings → Unified Library Location**.

## 🤖 Built-in Agents (24)

| Direct read | Junction (link) |
|-------------|-----------------|
| DeepSeek Harness · Codex · Cursor · Antigravity · Aider · Windsurf · Trae · GitHub Copilot · Devin · Amp | Claude · Cline · Roo Code · Gemini · OpenCode · Goose · Kiro · WorkBuddy · Qwen Code · Kilo Code · OpenHands · iFlow · Kimi · Grok |

> Any other agent can be added by editing `agents.json`.

## 📁 Project Structure

```
skill-nexus/
├── src/                        # Frontend (React + TS)
│   ├── App.tsx                 # Main UI
│   ├── AgentBadge.tsx          # Agent brand badge component
│   ├── agents.ts               # Agent brand color / logo mapping
│   ├── i18n.ts                 # zh / en dictionary
│   └── icons.tsx               # SVG icon set
└── src-tauri/                  # Backend (Rust)
    ├── src/core.rs             # Core logic: scan / link / sync / config
    ├── src/lib.rs              # Tauri command layer
    ├── Cargo.toml
    └── tauri.conf.json         # Window / bundling config
```

## 🙏 Credits

- [Agent Skills open specification](https://agentskills.io/specification) (SKILL.md)
- [simple-icons](https://simpleicons.org/) brand icon library
