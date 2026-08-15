<div align="center">

# SkillNexus

[**English**](README.en.md) | **简体中文**

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-1.97-orange?style=flat-square)
![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square)
![TypeScript](https://img.shields.io/badge/TypeScript-blue?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)

**一套技能库,接入所有 Agent。**

统一管理 AI 编码 Agent(Claude Code、Codex、Gemini CLI、DeepSeek Harness、Cursor、Cline ……)技能库的跨平台桌面工具。一个 `~/.agents/skills` 目录作为权威源,通过 junction / symlink 映射到各 Agent 的技能目录;原生支持 `~/.agents/skills` 的 Agent 则直接读取,无需任何链接。

</div>

---

## ⬇️ 下载

> 只想用,不想自己编译?直接下载最新版即可(Windows)。

| 文件 | 链接 |
|------|------|
| 📦 Windows 安装程序 | [SkillNexus_0.2.0_x64-setup.exe](https://github.com/Halface525/skill-nexus/releases/latest/download/SkillNexus_0.2.0_x64-setup.exe) |
| 🟢 Windows 免安装版 | [SkillNexus.exe](https://github.com/Halface525/skill-nexus/releases/latest/download/SkillNexus.exe) |

其他平台或更多版本:前往 [Releases](https://github.com/Halface525/skill-nexus/releases/latest)。

> 安装包仅 ~9MB,Win10/11 双击即装,无需任何运行环境。

---

## ✨ 功能特性

- **统一技能库** —— 所有技能集中存放于一个目录,一处管理,多处生效
- **一键同步** —— 为所有已安装的 Agent 自动创建链接,技能即装即用
- **智能检测** —— 命令在 PATH 上、或配置目录有真实数据才算已装(排除残留目录误判)
- **两级开关** —— 全局(某 Agent 是否使用任何技能)+ 每技能(某技能是否给某 Agent)
- **安装 / 卸载技能** —— 选择含 `SKILL.md` 的文件夹安装;一键卸载并移除所有链接
- **添加自定义 Agent** —— 内置 24 个主流 Agent;不在列表里的可手动添加,支持「手动标记已安装」
- **统一库可迁移** —— 首次运行引导选择库位置;库不在系统默认路径时,所有 Agent 均可逐个控制
- **SKILL.md 渲染** —— 详情面板完整渲染 Markdown(标题 / 代码块 / 表格 / 引用)
- **品牌徽章** —— 每个 Agent 以官方 logo / 品牌色徽章显示
- **关于面板** —— 版本信息 + 检查更新(对接 GitHub Release)
- **亮 / 暗 / 跟随系统** 主题 + **简体中文 / English** 双语界面

## 🗂️ 架构

所有 Agent 的技能都汇聚在一个**统一库**,两类接入方式:

| 类型 | 说明 | 代表 Agent |
|------|------|-----------|
| **direct(直接读取)** | Agent 原生支持读取 `~/.agents/skills`,无需链接 | DeepSeek Harness、Codex、Cursor、Trae、GitHub Copilot |
| **junction(链接)** | Agent 有固定的技能目录,通过 **junction**(Windows)/ **symlink**(macOS/Linux)指向统一库 | Claude、Cline、Gemini CLI、Qwen Code、Kiro |

> **库位置决定控制粒度**:统一库默认在 `~/.agents/skills`(多数 Agent 原生读取的位置)。首次运行可把它迁移到任意位置(如程序目录);一旦库不在约定路径,所有 Agent 都改由链接访问,即可逐个精细控制(含原本「直接读取」的 Agent)。

```
                ┌─────────────────────┐
                │   ~/.agents/skills   │  ← 统一技能库(权威源)
                └──────────┬──────────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
   junction/symlink        │          direct 读取
          │                │                │
   ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐
   │ ~/.claude/   │  │ ~/.gemini/  │  │ Codex /     │
   │  skills      │  │  skills     │  │ Cursor / …  │
   └─────────────┘  └─────────────┘  └─────────────┘
```

## 📸 截图

![主界面](docs/screenshot.png)

## 🖥️ 技术栈

- **桌面框架**:[Tauri 2](https://tauri.app/)(Rust 内核 + 系统 WebView)
- **后端**:Rust —— 目录扫描、junction/symlink 管理、SKILL.md 解析
- **前端**:[React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vite.dev/)
- **图标**:simple-icons 官方品牌图标 + 内联 SVG
- **Markdown 渲染**:[react-markdown](https://github.com/remarkjs/react-markdown)

## 📦 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| [Node.js](https://nodejs.org/) | ≥ 18 | 前端构建 |
| [Rust](https://www.rust-lang.org/) | ≥ 1.77 | 后端编译 |

安装 Tauri 平台依赖(按系统):

- **Windows**:Microsoft C++ Build Tools(含 C++ 工作负载)、[WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)(Win11 自带)
- **macOS**:Xcode Command Line Tools
- **Linux**:`webkit2gtk-4.1`、`libappindicator` 等,参见 [Tauri 前置依赖](https://tauri.app/start/prerequisites/)

## 🚀 快速开始

```bash
# 1. 克隆项目
git clone https://github.com/Halface525/skill-nexus.git
cd skill-nexus

# 2. 安装依赖
npm install

# 3. 开发模式运行(自动弹出桌面窗口)
npm run tauri dev
```

## 📦 打包发布

```bash
npm run tauri build
```

产物位于 `src-tauri/target/release/`:

- `bundle/nsis/*.exe` —— 安装程序(推荐分享)
- `SkillNexus.exe` —— 免安装绿色版

> 打包出的 exe 不依赖 Rust / Node 环境,对方双击即用(Win10/11 自带 WebView2)。

## ⚙️ 配置

### Agent 列表 `~/.agents/agents.json`

首次运行自动生成,可编辑增删任何 Agent,无需改代码:

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

| 字段 | 说明 |
|------|------|
| `name` | 显示名称 |
| `kind` | `direct` 直接读取统一库 / `junction` 建链接同步 |
| `dir` | junction 类必填:技能目录(相对 home),如 `.claude/skills` |
| `binary` | 检测用:可执行文件名(可选)。命令在 PATH 上是「已安装」的强证据 |
| `homeDir` | 检测用:home 下的配置目录(可选)。目录里有真实数据(不只有同步时创建的 `skills` 子目录)才视为已安装,可避免残留目录误判(如 `~/.gemini` 残留) |
| `appdata` | 检测用:`%APPDATA%` 下存在这些应用目录即视为已安装,如 `["Trae CN"]`(可选) |
| `enabled` | 使用开关(可选,默认 `true`)。`false` = 停止同步该 Agent 并移除其链接 |
| `manual` | 手动标记已安装(可选,默认 `false`)。`true` = 跳过检测,直接视为已装(自定义 Agent 用) |

> 应用内「扫描」面板可添加/停用自定义 Agent,无需手改此文件。

### 每技能排除 `~/.agents/skill_agents.json`

控制某个技能是否给某个 Agent(应用内「技能详情」面板操作):

```json
{
  "excluded": {
    "nature-writing": ["WorkBuddy"]
  }
}
```

语义:技能默认同步给所有已启用的 Agent;出现在排除列表里的不建链接。开启/关闭立即生效。

### 统一库位置 `~/.agents/settings.json`

```json
{
  "unifiedLibrary": "D:/MySkills",
  "librarySetup": true
}
```

- 首次运行会引导选择库位置(可迁移现有技能,源目录保留为 `.bak`)
- 库在系统默认的 `~/.agents/skills` 时,「直接读取」类 Agent 无法逐个控制;迁移到其他位置后即可全部精细控制
- 应用内「设置 → 统一库位置」随时更改(迁移式,自动同步)

## 🤖 内置 Agent(24 个)

| 直接读取(direct) | 需链接(junction) |
|------------------|------------------|
| DeepSeek Harness · Codex · Cursor · Antigravity · Aider · Windsurf · Trae · GitHub Copilot · Devin · Amp | Claude · Cline · Roo Code · Gemini · OpenCode · Goose · Kiro · WorkBuddy · Qwen Code · Kilo Code · OpenHands · iFlow · Kimi · Grok |

> 通过编辑 `agents.json` 可添加任意其他 Agent。

## 📁 项目结构

```
skill-nexus/
├── src/                        # 前端(React + TS)
│   ├── App.tsx                 # 主界面
│   ├── AgentBadge.tsx          # Agent 品牌徽章组件
│   ├── agents.ts               # Agent 品牌色 / logo 映射
│   ├── i18n.ts                 # 中英文词典
│   └── icons.tsx               # SVG 图标库
└── src-tauri/                  # 后端(Rust)
    ├── src/core.rs             # 核心逻辑:扫描 / 链接 / 同步 / 配置
    ├── src/lib.rs              # Tauri 命令层
    ├── Cargo.toml
    └── tauri.conf.json         # 窗口 / 打包配置
```

## 🙏 致谢

- [Agent Skills 开放标准](https://agentskills.io/specification)(SKILL.md)
- [simple-icons](https://simpleicons.org/) 品牌图标库
