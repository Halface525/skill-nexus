// ── 技能管理核心逻辑 ──────────────────────────────────
// 目录约定:
//   统一库      ~/.agents/skills        (所有技能的权威源)
//   Agent 配置  ~/.agents/agents.json   (可编辑,定义本机有哪些 agent)
//
// Agent 分两类:
//   direct   → 直接读取统一库,无需链接
//   junction → 有固定技能目录(如 ~/.claude/skills),需要建 junction 指向统一库

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

// Windows 上把路径分隔符统一成反斜杠。配置里的 dir 常用正斜杠(如 ".claude/skills"),
// 直接 home().join() 会产生混合分隔符(C:\...\.claude/skills),而 cmd/mklink 会把
// / 当作命令开关导致建链接失败。
fn norm_win(p: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(p.to_string_lossy().replace('/', "\\"))
    }
    #[cfg(not(windows))]
    {
        p
    }
}

pub fn unified_root() -> PathBuf {
    load_settings()
        .unified_library
        .map(|s| norm_win(PathBuf::from(s)))
        .unwrap_or_else(default_unified_root)
}

pub fn get_settings() -> SettingsView {
    let s = load_settings();
    SettingsView {
        unified_library: unified_root().to_string_lossy().to_string(),
        default_library: default_unified_root().to_string_lossy().to_string(),
        library_setup: s.library_setup,
        suggested_library: suggested_library_root().to_string_lossy().to_string(),
    }
}

// 建议的统一库位置:优先可写的程序目录(便携/绿色版),否则回退 APPDATA
fn suggested_library_root() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let p = dir.join("skills");
                let s = p.to_string_lossy().to_lowercase();
                // Program Files 目录不可写,回退到 APPDATA
                if !s.contains("program files") && !s.contains("programfiles") {
                    return norm_win(p);
                }
            }
        }
        std::env::var_os("APPDATA")
            .map(|a| norm_win(PathBuf::from(a).join("SkillNexus").join("skills")))
            .unwrap_or_else(default_unified_root)
    }
    #[cfg(not(windows))]
    {
        default_unified_root()
    }
}

fn write_settings(s: Settings) -> Result<(), String> {
    fs::create_dir_all(home().join(".agents")).map_err(|e| format!("无法创建配置目录: {e}"))?;
    let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
    fs::write(settings_path(), json).map_err(|e| format!("写入设置失败: {e}"))?;
    Ok(())
}

// 迁移统一库到新位置:复制技能 → 校验 → 源移走(.bak 保留)→ 更新设置。
// 源离开活动路径后,direct 类也变成靠链接访问,才能做到逐个控制。
pub fn migrate_library(target: &str) -> Result<(), String> {
    let target_p = norm_win(PathBuf::from(target));
    let source = unified_root();
    if source == target_p {
        // 相同位置:仅标记设置完成,不迁移
        return write_settings(Settings {
            unified_library: Some(target_p.to_string_lossy().to_string()),
            library_setup: true,
        });
    }
    fs::create_dir_all(&target_p).map_err(|e| format!("无法创建目录: {e}"))?;
    let mut copied = 0usize;
    if source.is_dir() {
        if let Ok(entries) = fs::read_dir(&source) {
            for e in entries.flatten() {
                if e.path().is_dir() {
                    let dst = target_p.join(e.file_name());
                    if dst.exists() {
                        // 清掉旧条目(可能是上次同步留下的过期链接),再复制
                        if is_link(&dst) {
                            let _ = remove_link(&dst);
                        } else {
                            let _ = fs::remove_dir_all(&dst);
                        }
                    }
                    copy_dir_all(&e.path(), &dst)?;
                    copied += 1;
                }
            }
        }
    }
    // 校验:新位置技能数与源一致,不一致则不动
    if copied > 0 {
        let src_count = count_dirs(&source);
        let dst_count = count_dirs(&target_p);
        if dst_count != src_count {
            return Err(format!(
                "迁移校验失败:源 {src_count} 个,新位置 {dst_count} 个,未改动"
            ));
        }
    }
    // 源移走保留(.bak,带时间戳防覆盖),活动路径留空 → 控制生效
    if copied > 0 && source.is_dir() {
        let mut bak = source.with_extension("bak");
        if bak.exists() {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            bak = PathBuf::from(format!("{}.{}.bak", source.to_string_lossy(), ts));
        }
        fs::rename(&source, &bak).map_err(|e| format!("源目录移走失败: {e}"))?;
    }
    // 确保约定路径存在,供 direct 组按开关建链接
    let _ = fs::create_dir_all(&convention_root());
    write_settings(Settings {
        unified_library: Some(target_p.to_string_lossy().to_string()),
        library_setup: true,
    })
}

// 设置统一库位置(迁移式:复制技能 + 源移走)
pub fn set_unified_library(path: &str) -> Result<(), String> {
    migrate_library(path)
}

// ── 设置(settings.json)───────────────────────────────
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub unified_library: Option<String>,
    pub library_setup: bool, // 是否已完成首次库位置设置
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub unified_library: String,
    pub default_library: String,
    pub library_setup: bool,
    pub suggested_library: String,
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

// ── 每个技能排除的 agent(per-skill 管理)────────────
// ~/.agents/skill_agents.json:
//   { "excluded": { "nature-writing": ["WorkBuddy"] } }
// 语义:技能默认同步给所有已启用的 junction 类 agent,排除列表里的不建链接
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SkillAgents {
    excluded: HashMap<String, Vec<String>>,
}

fn skill_agents_path() -> PathBuf {
    home().join(".agents").join("skill_agents.json")
}

fn load_exclusions() -> HashMap<String, Vec<String>> {
    if let Ok(raw) = fs::read_to_string(skill_agents_path()) {
        if let Ok(s) = serde_json::from_str::<SkillAgents>(&raw) {
            return s.excluded;
        }
    }
    HashMap::new()
}

fn excluded_agents(skill: &str) -> Vec<String> {
    load_exclusions()
        .get(skill)
        .cloned()
        .unwrap_or_default()
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
    #[serde(default = "default_enabled")]
    pub enabled: bool, // 使用开关(junction 类):false = 停止同步并移除链接
    #[serde(default)]
    pub manual: bool, // 手动标记已安装(自定义 agent 用,不自动检测)
}

fn default_enabled() -> bool {
    true
}

impl AgentConfig {
    // home 配置目录里是否有真实数据:
    // 排除 SkillNexus 同步时自己创建的 skills 子目录,目录里还有别的内容才算真装了。
    //   ~/.gemini   → 只有 skills(同步残留)→ 不算
    //   ~/.workbuddy → db / 配置 / sessions 等真实数据 → 算
    fn home_dir_real(&self) -> bool {
        self.home_dir.as_deref().map(|h| {
            let p = norm_win(home().join(h));
            if !p.is_dir() {
                return false;
            }
            fs::read_dir(&p)
                .map(|it| it.flatten().any(|e| e.file_name() != "skills"))
                .unwrap_or(false)
        }).unwrap_or(false)
    }

    // %APPDATA% 下是否存在应用目录(GUI 应用安装时创建)
    fn appdata_exists(&self) -> bool {
        self.appdata.iter().any(|d| {
            std::env::var_os("APPDATA")
                .map(|a| PathBuf::from(a).join(d).exists())
                .unwrap_or(false)
        })
    }

    // 该 agent 是否在本机检测到(决定是否显示在卡片、是否参与同步)
    fn detected(&self) -> bool {
        // 手动标记已安装 → 直接算
        if self.manual {
            return true;
        }
        // ① 命令在 PATH 上 → 强证据,必定已安装
        if self.binary.as_deref().map(on_path).unwrap_or(false) {
            return true;
        }
        // ② 否则看配置目录是否有真实数据 / AppData 是否有应用目录。
        //    home 目录只有同步残留的 skills 时不算,避免 ~/.gemini 这类误判
        self.home_dir_real() || self.appdata_exists()
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
        AgentConfig { name: "DeepSeek Harness".into(), kind: "direct".into(), dir: None, binary: Some("dsh".into()), home_dir: Some(".dsh".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Codex".into(), kind: "direct".into(), dir: None, binary: Some("codex".into()), home_dir: Some(".codex".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Cursor".into(), kind: "direct".into(), dir: None, binary: Some("cursor".into()), home_dir: None, appdata: vec!["Cursor".into()], enabled: true, manual: false },
        AgentConfig { name: "Antigravity".into(), kind: "direct".into(), dir: None, binary: Some("antigravity".into()), home_dir: None, appdata: vec!["Antigravity".into()], enabled: true, manual: false },
        AgentConfig { name: "Aider".into(), kind: "direct".into(), dir: None, binary: Some("aider".into()), home_dir: Some(".aider".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Windsurf".into(), kind: "direct".into(), dir: None, binary: Some("windsurf".into()), home_dir: None, appdata: vec!["Windsurf".into()], enabled: true, manual: false },
        AgentConfig { name: "Trae".into(), kind: "direct".into(), dir: None, binary: Some("trae".into()), home_dir: Some(".trae".into()), appdata: vec!["Trae CN".into(), "Trae".into()], enabled: true, manual: false },
        AgentConfig { name: "GitHub Copilot".into(), kind: "direct".into(), dir: None, binary: Some("github-copilot".into()), home_dir: None, appdata: vec!["Copilot".into()], enabled: true, manual: false },
        AgentConfig { name: "Devin".into(), kind: "direct".into(), dir: None, binary: Some("devin".into()), home_dir: Some(".devin".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Amp".into(), kind: "direct".into(), dir: None, binary: Some("amp".into()), home_dir: None, appdata: vec![], enabled: true, manual: false },
        // ── 固定技能目录、需要 junction 同步的 agent ──
        AgentConfig { name: "Claude".into(), kind: "junction".into(), dir: Some(".claude/skills".into()), binary: Some("claude".into()), home_dir: Some(".claude".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Cline".into(), kind: "junction".into(), dir: Some(".cline/skills".into()), binary: None, home_dir: Some(".cline".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Roo Code".into(), kind: "junction".into(), dir: Some(".roo/skills".into()), binary: None, home_dir: Some(".roo".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Gemini".into(), kind: "junction".into(), dir: Some(".gemini/skills".into()), binary: Some("gemini".into()), home_dir: Some(".gemini".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "OpenCode".into(), kind: "junction".into(), dir: Some(".config/opencode/skills".into()), binary: Some("opencode".into()), home_dir: None, appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Goose".into(), kind: "junction".into(), dir: Some(".config/goose/skills".into()), binary: Some("goose".into()), home_dir: Some(".config/goose".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Kiro".into(), kind: "junction".into(), dir: Some(".kiro/skills".into()), binary: Some("kiro".into()), home_dir: Some(".kiro".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "WorkBuddy".into(), kind: "junction".into(), dir: Some(".workbuddy/skills".into()), binary: Some("wb".into()), home_dir: Some(".workbuddy".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Qwen Code".into(), kind: "junction".into(), dir: Some(".qwen/skills".into()), binary: Some("qwen-code".into()), home_dir: Some(".qwen".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Kilo Code".into(), kind: "junction".into(), dir: Some(".kilocode/skills".into()), binary: None, home_dir: Some(".kilocode".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "OpenHands".into(), kind: "junction".into(), dir: Some(".openhands/skills".into()), binary: Some("openhands".into()), home_dir: Some(".openhands".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "iFlow".into(), kind: "junction".into(), dir: Some(".iflow/skills".into()), binary: Some("iflow".into()), home_dir: Some(".iflow".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Kimi".into(), kind: "junction".into(), dir: Some(".kimi/skills".into()), binary: Some("kimi".into()), home_dir: Some(".kimi".into()), appdata: vec![], enabled: true, manual: false },
        AgentConfig { name: "Grok".into(), kind: "junction".into(), dir: Some(".grok/skills".into()), binary: Some("grok".into()), home_dir: Some(".grok".into()), appdata: vec![], enabled: true, manual: false },
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

// 约定的直接读取路径(agentskills 规范:各 agent 默认读这里)
fn convention_root() -> PathBuf {
    norm_win(home().join(".agents").join("skills"))
}

// 统一库是否就在约定路径上。
// 是 → direct 类直接读库(无法逐个控制,只能一组);
// 否 → direct 类也要靠约定路径里的链接访问(可逐个控制)
fn library_is_at_convention() -> bool {
    unified_root() == convention_root()
}

fn agent_skills_root(a: &AgentConfig) -> PathBuf {
    if a.kind == "junction" {
        a.dir
            .as_ref()
            .map(|d| norm_win(home().join(d)))
            .unwrap_or_else(|| norm_win(home().join(format!(".{}/skills", a.name.to_lowercase()))))
    } else {
        convention_root() // direct 类读约定的 ~/.agents/skills 路径
    }
}

// ── 数据模型 ────────────────────────────────────────
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub name: String,
    pub kind: String, // "direct" 直接读取 | "junction" 链接
    pub ok: bool,
    pub excluded: bool, // 该技能是否被此 agent 排除(per-skill 开关)
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
    let excluded = excluded_agents(skill_name);
    let direct_free = library_is_at_convention(); // 库在约定路径 → direct 直接读
    load_agents()
        .iter()
        .filter(|a| a.detected() && a.enabled) // 只显示本机已安装且启用的 agent
        .map(|a| {
            let is_ex = excluded.iter().any(|e| e == &a.name);
            let ok = if a.kind == "direct" && direct_free {
                unified_exists // 直接读统一库,库在即接入
            } else {
                !is_ex && is_link(&agent_skills_root(a).join(skill_name)) // 建链接判定
            };
            AgentStatus { name: a.name.clone(), kind: a.kind.clone(), ok, excluded: is_ex }
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

// 移除单个链接(Windows junction / Unix symlink 都不跟随目标)
fn remove_link(link: &Path) -> Result<(), String> {
    if is_link(link) {
        fs::remove_dir(link).map_err(|e| format!("移除链接失败: {e}"))?;
    }
    Ok(())
}

// 为单个技能在所有 agent 目录建链接(跳过被该技能排除的 agent)。
// 库在约定路径时,direct 类直接读库无需链接;库在别处时,direct 也靠链接访问。
fn link_all(name: &str, target: &Path) -> (usize, usize) {
    let mut ok = 0usize;
    let mut fail = 0usize;
    let excluded = excluded_agents(name);
    let direct_free = library_is_at_convention();
    for a in load_agents() {
        if !a.detected() || !a.enabled {
            continue;
        }
        if a.kind == "direct" && direct_free {
            continue; // direct 直接读库,无需链接
        }
        if excluded.iter().any(|e| e == &a.name) {
            continue; // 该技能被此 agent 排除
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

pub fn uninstall_skill(name: &str, lang: &str) -> Result<(), String> {
    // 名称校验同时保证无法路径穿越(只允许小写字母数字 + -)
    if !is_valid_name(name) {
        return Err(format!(
            "{}: \"{name}\"",
            tr(lang, "技能名格式不对", "Invalid skill name")
        ));
    }
    let target = unified_root().join(name);
    if !target.is_dir() {
        return Err(format!(
            "{}: \"{name}\"",
            tr(lang, "技能不存在", "Skill not found")
        ));
    }
    // 先尽力移除所有 junction 类 agent 目录里的链接,再删除统一库中的技能
    for a in load_agents() {
        if a.kind == "junction" {
            let _ = remove_link(&agent_skills_root(&a).join(name));
        }
    }
    fs::remove_dir_all(&target).map_err(|e| format!("{}: {e}", tr(lang, "删除技能失败", "Failed to delete skill")))?;
    Ok(())
}

// 切换某个 agent 的使用开关。关掉 junction 类 agent 时,一并移除它目录里
// 指向统一库的链接(可逆:重新打开 + 一键同步即可恢复)。
pub fn set_agent_enabled(name: &str, enabled: bool, lang: &str) -> Result<(), String> {
    let p = agents_config_path();
    let mut agents = load_agents();
    let mut found = false;
    let unified_canon = fs::canonicalize(&unified_root()).unwrap_or_else(|_| unified_root());
    for a in agents.iter_mut() {
        if a.name != name {
            continue;
        }
        if !enabled && a.kind == "junction" {
            let root = agent_skills_root(a);
            if let Ok(entries) = fs::read_dir(&root) {
                for e in entries.flatten() {
                    let path = e.path();
                    // 只删指向统一库的链接,别的 junction 不动
                    if let Ok(resolved) = fs::canonicalize(&path) {
                        if resolved.starts_with(&unified_canon) {
                            let _ = remove_link(&path);
                        }
                    }
                }
            }
        }
        a.enabled = enabled;
        found = true;
    }
    if !found {
        return Err(tr(lang, "未找到该 Agent", "Agent not found"));
    }
    let json = serde_json::to_string_pretty(&Config { agents }).map_err(|e| e.to_string())?;
    fs::write(&p, json).map_err(|e| format!("{}: {e}", tr(lang, "写入配置失败", "Failed to write config")))?;
    Ok(())
}

// 设置某个技能是否被某 agent 使用:enabled=true 启用并立即建链接;
// false 排除该 agent 并立即移除该技能的链接。
pub fn set_skill_agent(skill: &str, agent: &str, enabled: bool, lang: &str) -> Result<(), String> {
    if !is_valid_name(skill) {
        return Err(format!(
            "{}: \"{skill}\"",
            tr(lang, "技能名格式不对", "Invalid skill name")
        ));
    }
    let mut ex = load_exclusions();
    let list = ex.entry(skill.to_string()).or_default();
    if enabled {
        list.retain(|a| a != agent);
        if list.is_empty() {
            ex.remove(skill);
        }
        if let Some(a) = load_agents().iter().find(|a| a.name == agent) {
            if a.kind == "junction" && a.detected() && a.enabled {
                let target = unified_root().join(skill);
                if target.is_dir() {
                    let _ = make_link(&agent_skills_root(a).join(skill), &target);
                }
            }
        }
    } else {
        if !list.iter().any(|a| a == agent) {
            list.push(agent.to_string());
        }
        if let Some(a) = load_agents().iter().find(|a| a.name == agent) {
            let _ = remove_link(&agent_skills_root(a).join(skill));
        }
    }
    let json =
        serde_json::to_string_pretty(&SkillAgents { excluded: ex }).map_err(|e| e.to_string())?;
    fs::write(skill_agents_path(), json)
        .map_err(|e| format!("{}: {e}", tr(lang, "写入配置失败", "Failed to write config")))?;
    Ok(())
}

// ── 添加自定义 Agent ──────────────────────────────
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAgent {
    pub name: String,
    pub kind: String, // "direct" | "junction"
    pub dir: Option<String>,
    pub binary: Option<String>,
    pub home_dir: Option<String>,
    pub appdata: Vec<String>,
    pub manual: bool,
}

pub fn add_agent(agent: NewAgent, lang: &str) -> Result<String, String> {
    let name = agent.name.trim().to_string();
    if name.is_empty() {
        return Err(tr(lang, "Agent 名称不能为空", "Agent name cannot be empty"));
    }
    if agent.kind != "direct" && agent.kind != "junction" {
        return Err(tr(lang, "类型只能是 direct 或 junction", "Kind must be direct or junction"));
    }
    if agent.kind == "junction"
        && agent.dir.as_deref().map(|d| d.trim()).unwrap_or("").is_empty()
    {
        return Err(tr(lang, "junction 类需要填写技能目录", "Junction agents need a skill directory"));
    }
    let mut agents = load_agents();
    if agents.iter().any(|a| a.name.eq_ignore_ascii_case(&name)) {
        return Err(format!("{}: \"{name}\"", tr(lang, "已存在同名 Agent", "An agent with this name already exists")));
    }
    let clean = |s: Option<String>| s.map(|x| x.trim().to_string()).filter(|x| !x.is_empty());
    agents.push(AgentConfig {
        name: name.clone(),
        kind: agent.kind.clone(),
        dir: clean(agent.dir),
        binary: clean(agent.binary),
        home_dir: clean(agent.home_dir),
        appdata: agent.appdata.iter().map(|d| d.trim().to_string()).filter(|d| !d.is_empty()).collect(),
        enabled: true,
        manual: agent.manual,
    });
    let json = serde_json::to_string_pretty(&Config { agents }).map_err(|e| e.to_string())?;
    fs::write(agents_config_path(), json).map_err(|e| format!("{}: {e}", tr(lang, "写入配置失败", "Failed to write config")))?;
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
    pub enabled: bool, // 使用开关(junction 类)
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
    let direct_free = library_is_at_convention();
    let mut agents = Vec::new();
    for a in load_agents() {
        if !a.detected() {
            continue; // 只报告本机已安装的 agent
        }
        let root = agent_skills_root(&a);
        let synced_count = if a.kind == "direct" && direct_free {
            unified_count // 库在约定路径,direct 直接读全部
        } else {
            count_synced(&root, &unified)
        };
        agents.push(AgentScan {
            name: a.name,
            installed: true,
            enabled: a.enabled,
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





