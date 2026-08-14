// ── 技能管理核心逻辑 ──────────────────────────────────
// 目录约定:
//   统一库      ~/.agents/skills        (所有技能的权威源)
//   Agent 配置  ~/.agents/agents.json   (可编辑,定义本机有哪些 agent)
//
// Agent 分两类:
//   direct   → 直接读取统一库,无需链接
//   junction → 有固定技能目录(如 ~/.claude/skills),需要建 junction 指向统一库

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ── 路径 ────────────────────────────────────────────
fn home() -> PathBuf {
    #[cfg(windows)]
    let h = std::env::var("USERPROFILE").ok();
    #[cfg(unix)]
    let h = std::env::var("HOME").ok();
    h.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

pub fn unified_root() -> PathBuf {
    load_settings()
        .unified_library
        .map(PathBuf::from)
        .unwrap_or_else(default_unified_root)
}

pub fn get_settings() -> SettingsView {
    SettingsView {
        unified_library: unified_root().to_string_lossy().to_string(),
        default_library: default_unified_root().to_string_lossy().to_string(),
    }
}

pub fn set_unified_library(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    fs::create_dir_all(p).map_err(|e| format!("无法创建目录: {e}"))?;
    fs::create_dir_all(home().join(".agents")).map_err(|e| format!("无法创建配置目录: {e}"))?;
    let s = Settings { unified_library: Some(path.to_string()) };
    let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
    fs::write(settings_path(), json).map_err(|e| format!("写入设置失败: {e}"))?;
    Ok(())
}

// ── 设置(settings.json)───────────────────────────────
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub unified_library: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub unified_library: String,
    pub default_library: String,
}

fn settings_path() -> PathBuf {
    home().join(".agents").join("settings.json")
}

fn load_settings() -> Settings {
    if let Ok(raw) = fs::read_to_string(settings_path()) {
        if let Ok(s) = serde_json::from_str::<Settings>(&raw) {
            return s;
        }
    }
    Settings::default()
}

fn default_unified_root() -> PathBuf {
    home().join(".agents").join("skills")
}

fn agents_config_path() -> PathBuf {
    home().join(".agents").join("agents.json")
}

// ── Agent 配置(可编辑)────────────────────────────────
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub name: String,
    pub kind: String, // "direct" 直接读取 | "junction" 链接
    #[serde(default)]
    pub dir: Option<String>, // junction 类: 相对 home 的技能目录,如 ".claude/skills"
    #[serde(default)]
    pub binary: Option<String>, // 检测用: 可执行文件名,如 "claude"
    #[serde(default)]
    pub home_dir: Option<String>, // 检测用: home 下存在该目录即视为已安装,如 ".claude"
    #[serde(default)]
    pub appdata: Vec<String>, // 检测用: %APPDATA% 下存在这些目录即视为已安装,如 ["Trae CN"]
}

impl AgentConfig {
    // 该 agent 是否在本机检测到(决定是否显示在卡片、是否参与同步)
    fn detected(&self) -> bool {
        self.binary.as_deref().map(on_path).unwrap_or(false)
            || self.home_dir.as_deref().map(|h| home().join(h).exists()).unwrap_or(false)
            || self.appdata.iter().any(|d| {
                std::env::var_os("APPDATA")
                    .map(|a| PathBuf::from(a).join(d).exists())
                    .unwrap_or(false)
            })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    agents: Vec<AgentConfig>,
}

fn default_agents() -> Vec<AgentConfig> {
    vec![
        // ── 直接读取统一库(~/.agents/skills)的 agent ── 无需同步
        AgentConfig { name: "DeepSeek Harness".into(), kind: "direct".into(), dir: None, binary: Some("dsh".into()), home_dir: Some(".dsh".into()), appdata: vec![] },
        AgentConfig { name: "Codex".into(), kind: "direct".into(), dir: None, binary: Some("codex".into()), home_dir: Some(".codex".into()), appdata: vec![] },
        AgentConfig { name: "Cursor".into(), kind: "direct".into(), dir: None, binary: Some("cursor".into()), home_dir: None, appdata: vec!["Cursor".into()] },
        AgentConfig { name: "Antigravity".into(), kind: "direct".into(), dir: None, binary: Some("antigravity".into()), home_dir: None, appdata: vec!["Antigravity".into()] },
        AgentConfig { name: "Aider".into(), kind: "direct".into(), dir: None, binary: Some("aider".into()), home_dir: Some(".aider".into()), appdata: vec![] },
        AgentConfig { name: "Windsurf".into(), kind: "direct".into(), dir: None, binary: Some("windsurf".into()), home_dir: None, appdata: vec!["Windsurf".into()] },
        AgentConfig { name: "Trae".into(), kind: "direct".into(), dir: None, binary: Some("trae".into()), home_dir: Some(".trae".into()), appdata: vec!["Trae CN".into(), "Trae".into()] },
        AgentConfig { name: "GitHub Copilot".into(), kind: "direct".into(), dir: None, binary: Some("github-copilot".into()), home_dir: Some(".github".into()), appdata: vec!["Copilot".into()] },
        AgentConfig { name: "Devin".into(), kind: "direct".into(), dir: None, binary: Some("devin".into()), home_dir: Some(".devin".into()), appdata: vec![] },
        AgentConfig { name: "Amp".into(), kind: "direct".into(), dir: None, binary: Some("amp".into()), home_dir: Some(".config/agents".into()), appdata: vec![] },
        // ── 固定技能目录、需要 junction 同步的 agent ──
        AgentConfig { name: "Claude".into(), kind: "junction".into(), dir: Some(".claude/skills".into()), binary: Some("claude".into()), home_dir: Some(".claude".into()), appdata: vec![] },
        AgentConfig { name: "Cline".into(), kind: "junction".into(), dir: Some(".cline/skills".into()), binary: None, home_dir: Some(".cline".into()), appdata: vec![] },
        AgentConfig { name: "Roo Code".into(), kind: "junction".into(), dir: Some(".roo/skills".into()), binary: None, home_dir: Some(".roo".into()), appdata: vec![] },
        AgentConfig { name: "Gemini".into(), kind: "junction".into(), dir: Some(".gemini/skills".into()), binary: Some("gemini".into()), home_dir: Some(".gemini".into()), appdata: vec![] },
        AgentConfig { name: "OpenCode".into(), kind: "junction".into(), dir: Some(".config/opencode/skills".into()), binary: Some("opencode".into()), home_dir: None, appdata: vec![] },
        AgentConfig { name: "Goose".into(), kind: "junction".into(), dir: Some(".config/goose/skills".into()), binary: Some("goose".into()), home_dir: Some(".config/goose".into()), appdata: vec![] },
        AgentConfig { name: "Kiro".into(), kind: "junction".into(), dir: Some(".kiro/skills".into()), binary: Some("kiro".into()), home_dir: Some(".kiro".into()), appdata: vec![] },
        AgentConfig { name: "WorkBuddy".into(), kind: "junction".into(), dir: Some(".workbuddy/skills".into()), binary: Some("wb".into()), home_dir: Some(".workbuddy".into()), appdata: vec![] },
        AgentConfig { name: "Qwen Code".into(), kind: "junction".into(), dir: Some(".qwen/skills".into()), binary: Some("qwen-code".into()), home_dir: Some(".qwen".into()), appdata: vec![] },
        AgentConfig { name: "Kilo Code".into(), kind: "junction".into(), dir: Some(".kilocode/skills".into()), binary: None, home_dir: Some(".kilocode".into()), appdata: vec![] },
        AgentConfig { name: "OpenHands".into(), kind: "junction".into(), dir: Some(".openhands/skills".into()), binary: Some("openhands".into()), home_dir: Some(".openhands".into()), appdata: vec![] },
        AgentConfig { name: "iFlow".into(), kind: "junction".into(), dir: Some(".iflow/skills".into()), binary: Some("iflow".into()), home_dir: Some(".iflow".into()), appdata: vec![] },
        AgentConfig { name: "Kimi".into(), kind: "junction".into(), dir: Some(".kimi/skills".into()), binary: Some("kimi".into()), home_dir: Some(".kimi".into()), appdata: vec![] },
        AgentConfig { name: "Grok".into(), kind: "junction".into(), dir: Some(".grok/skills".into()), binary: Some("grok".into()), home_dir: Some(".grok".into()), appdata: vec![] },
    ]
}

// 读取配置;文件不存在则写入默认配置(方便用户编辑添加 agent)
fn load_agents() -> Vec<AgentConfig> {
    let p = agents_config_path();
    if let Ok(raw) = fs::read_to_string(&p) {
        if let Ok(cfg) = serde_json::from_str::<Config>(&raw) {
            if !cfg.agents.is_empty() {
                return cfg.agents;
            }
        }
    } else {
        if let Ok(json) = serde_json::to_string_pretty(&Config { agents: default_agents() }) {
            let _ = fs::create_dir_all(home().join(".agents"));
            let _ = fs::write(&p, json);
        }
    }
    default_agents()
}

fn agent_skills_root(a: &AgentConfig) -> PathBuf {
    if a.kind == "junction" {
        a.dir
            .as_ref()
            .map(|d| home().join(d))
            .unwrap_or_else(|| home().join(format!(".{}/skills", a.name.to_lowercase())))
    } else {
        unified_root() // direct 类直接读统一库
    }
}

// ── 数据模型 ────────────────────────────────────────
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub name: String,
    pub kind: String, // "direct" 直接读取 | "junction" 链接
    pub ok: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub dir: String,
    pub has_skill_md: bool,
    pub agents: Vec<AgentStatus>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub ok: usize,
    pub fail: usize,
    pub total: usize,
}

fn build_agent_statuses(skill_name: &str) -> Vec<AgentStatus> {
    let unified_exists = unified_root().is_dir();
    load_agents()
        .iter()
        .filter(|a| a.detected()) // 只显示本机已安装的 agent
        .map(|a| {
            let ok = match a.kind.as_str() {
                "direct" => unified_exists, // 直接读统一库,库在即接入
                _ => is_link(&agent_skills_root(a).join(skill_name)), // junction 是否已建
            };
            AgentStatus { name: a.name.clone(), kind: a.kind.clone(), ok }
        })
        .collect()
}

// ── 链接:Windows junction / Unix symlink ────────────
// Rust 标准库把 Windows 的 junction 也判定为 is_symlink(),所以两端共用一份检查
fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn make_link(link: &Path, target: &Path) -> Result<(), String> {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    match fs::symlink_metadata(link) {
        Ok(_) => {
            if is_link(link) {
                return Ok(()); // 已是链接,幂等
            }
            fs::remove_dir_all(link).map_err(|e| format!("移除已有目录失败: {e}"))?;
        }
        Err(_) => {}
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let status = std::process::Command::new("cmd")
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW,避免闪黑框
            .args(["/c", "mklink", "/J"])
            .arg(link.as_os_str())
            .arg(target.as_os_str())
            .status()
            .map_err(|e| format!("调用 mklink 失败: {e}"))?;
        if !status.success() {
            return Err(format!("mklink 失败(code {:?})", status.code()));
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| format!("创建 symlink 失败: {e}"))?;
    }
    Ok(())
}

// 为单个技能在所有 junction 类 agent 目录建链接
fn link_all(name: &str, target: &Path) -> (usize, usize) {
    let mut ok = 0usize;
    let mut fail = 0usize;
    for a in load_agents() {
        if a.kind != "junction" || !a.detected() {
            continue; // 只同步本机已安装的 junction 类 agent
        }
        if make_link(&agent_skills_root(&a).join(name), target).is_ok() {
            ok += 1;
        } else {
            fail += 1;
        }
    }
    (ok, fail)
}

// ── SKILL.md 解析 ───────────────────────────────────
fn parse_description(raw: &str) -> String {
    for line in raw.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("description:") {
            return rest.trim().to_string();
        }
    }
    String::new()
}

fn parse_name(raw: &str, fallback: &str) -> String {
    for line in raw.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("name:") {
            let n = rest.trim().to_string();
            if !n.is_empty() {
                return n;
            }
        }
    }
    fallback.to_string()
}

// 等效于正则 ^[a-z0-9]+(-[a-z0-9]+)*$
fn is_valid_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut prev_dash = false;
    for c in name.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            prev_dash = false;
        } else if c == '-' {
            if prev_dash {
                return false; // 连续 -
            }
            prev_dash = true;
        } else {
            return false;
        }
    }
    !prev_dash // 不能以 - 结尾
}

// ── 文件操作 ────────────────────────────────────────
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| format!("复制 {} 失败: {e}", from.display()))?;
        }
    }
    Ok(())
}

// ── 对外接口(被 tauri 命令调用)──────────────────────
pub fn load_skills() -> Result<Vec<Skill>, String> {
    let root = unified_root();
    let mut skills = Vec::new();
    if !root.is_dir() {
        return Ok(skills);
    }
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let dir = entry.path();
        let md = dir.join("SKILL.md");
        let mut description = String::new();
        let has_skill_md = md.is_file();
        if has_skill_md {
            if let Ok(raw) = fs::read_to_string(&md) {
                description = parse_description(&raw);
            }
        }
        skills.push(Skill {
            name: name.clone(),
            description,
            dir: dir.to_string_lossy().to_string(),
            has_skill_md,
            agents: build_agent_statuses(&name),
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

// 简单双语(zh/en)文案
fn tr(lang: &str, zh: &str, en: &str) -> String {
    if lang == "en" { en.to_string() } else { zh.to_string() }
}

pub fn read_skill_md(dir: &str, lang: &str) -> Result<String, String> {
    let p = Path::new(dir).join("SKILL.md");
    fs::read_to_string(&p)
        .map_err(|e| format!("{}: {e}", tr(lang, "读取 SKILL.md 失败", "Failed to read SKILL.md")))
}

pub fn install_skill(src: &str, lang: &str) -> Result<String, String> {
    let src_path = Path::new(src);
    let md = src_path.join("SKILL.md");
    if !md.is_file() {
        return Err(tr(lang, "所选文件夹里没有 SKILL.md", "No SKILL.md in the selected folder"));
    }
    let raw = fs::read_to_string(&md)
        .map_err(|e| format!("{}: {e}", tr(lang, "读取 SKILL.md 失败", "Failed to read SKILL.md")))?;
    let fallback = src_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let name = parse_name(&raw, &fallback);
    if !is_valid_name(&name) {
        return Err(format!(
            "{}: \"{name}\"",
            tr(lang, "技能名格式不对(要求小写字母数字,可用 - 连接)", "Invalid skill name (lowercase letters/digits, dash-separated)")
        ));
    }
    let target = unified_root().join(&name);
    if target.exists() {
        return Err(format!(
            "{}: \"{name}\"",
            tr(lang, "统一库里已存在同名技能", "A skill with this name already exists")
        ));
    }
    fs::create_dir_all(unified_root()).map_err(|e| e.to_string())?;
    copy_dir_all(&src_path, &target)?;
    // UTF-8 无 BOM 重写 SKILL.md(源文件可能带 BOM)
    fs::write(target.join("SKILL.md"), raw.as_bytes())
        .map_err(|e| format!("写入 SKILL.md 失败: {e}"))?;
    let (_ok, _fail) = link_all(&name, &target);
    Ok(name)
}

pub fn sync_all() -> Result<SyncResult, String> {
    let root = unified_root();
    let mut ok = 0usize;
    let mut fail = 0usize;
    let mut total = 0usize;
    if !root.is_dir() {
        return Ok(SyncResult { ok: 0, fail: 0, total: 0 });
    }
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.path().join("SKILL.md").is_file() {
            fail += 1;
            continue;
        }
        total += 1;
        let (_, link_fail) = link_all(&name, &entry.path());
        if link_fail > 0 {
            fail += 1;
        } else {
            ok += 1;
        }
    }
    Ok(SyncResult { ok, fail, total })
}

// ── 扫描检测 ────────────────────────────────────────
fn count_dirs(p: &Path) -> usize {
    if !p.is_dir() {
        return 0;
    }
    fs::read_dir(p)
        .map(|it| it.flatten().filter(|e| e.path().is_dir()).count())
        .unwrap_or(0)
}

fn on_path(name: &str) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .any(|dir| dir.join(name).is_file() || dir.join(format!("{name}.exe")).is_file())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentScan {
    pub name: String,
    pub installed: bool,
    pub kind: String, // "direct" 直接读取 | "junction" 链接
    pub root: String,
    pub skill_count: usize,
    pub synced_count: usize, // 已同步(已建链接)的技能数
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanInfo {
    pub unified_root: String,
    pub unified_count: usize,
    pub agents: Vec<AgentScan>,
}

// 统计某 junction agent 目录下,统一库里有几个技能已建链接
fn count_synced(agent_root: &Path, unified: &Path) -> usize {
    let mut n = 0usize;
    if let Ok(entries) = fs::read_dir(unified) {
        for e in entries.flatten() {
            if e.path().is_dir() && is_link(&agent_root.join(e.file_name())) {
                n += 1;
            }
        }
    }
    n
}

pub fn scan_info() -> ScanInfo {
    let unified = unified_root();
    let unified_count = count_dirs(&unified);
    let mut agents = Vec::new();
    for a in load_agents() {
        if !a.detected() {
            continue; // 只报告本机已安装的 agent
        }
        let root = agent_skills_root(&a);
        let synced_count = if a.kind == "junction" {
            count_synced(&root, &unified)
        } else {
            unified_count // 直接读统一库,库在即全部接入
        };
        agents.push(AgentScan {
            name: a.name,
            installed: true,
            kind: a.kind,
            root: root.to_string_lossy().to_string(),
            skill_count: count_dirs(&root),
            synced_count,
        });
    }
    ScanInfo {
        unified_root: unified.to_string_lossy().to_string(),
        unified_count,
        agents,
    }
}

pub fn open_dir(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(path).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}
