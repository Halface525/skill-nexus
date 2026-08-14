// ── 国际化(zh / en)──────────────────────────────────
export type Lang = "zh" | "en";

const messages: Record<Lang, Record<string, string>> = {
  zh: {
    "app.title": "SkillNexus",
    "app.subtitle": "统一管理各 Agent 技能",
    "btn.install": "安装技能",
    "btn.sync": "一键同步",
    "btn.scan": "扫描",
    "btn.settings": "设置",
    "theme.toLight": "切换到亮色",
    "theme.toDark": "切换到暗色",
    "search.placeholder": "搜索技能...",
    "list.empty": "技能库是空的,点「安装技能」添加第一个技能",
    "list.noMatch": "没有匹配的技能",
    "status.count": "{n} 个技能 | 统一库: {path}",
    "status.loadFailed": "加载失败",
    "agent.connected": "已接入",
    "agent.notSynced": "未同步",
    "detail.noDesc": "无描述",
    "detail.select": "选择左侧一个技能查看详情",
    "detail.openFolder": "打开文件夹",
    "detail.skillMd": "SKILL.md",
    "detail.noMd": "(无 SKILL.md 或读取失败)",
    "scan.title": "扫描报告",
    "scan.unified": "统一技能库",
    "scan.skills": "{n} 个技能",
    "scan.agents": "本机 Agent 检测",
    "scan.installed": "已安装",
    "scan.kind.link": "链接",
    "scan.kind.direct": "直接读取",
    "scan.synced": "已同步 {n}/{total}",
    "scan.connected": "已接入",
    "scan.path.unified": "(统一库)",
    "scan.hint": "想添加其他 Agent?编辑 ~/.agents/agents.json 后点扫描刷新。",
    "scan.close": "关闭",
    "toast.installed": "技能 \"{name}\" 安装成功",
    "toast.installFailed": "安装失败",
    "toast.syncDone": "同步完成:{ok} 成功,{fail} 失败",
    "toast.syncFailed": "同步失败",
    "settings.title": "设置",
    "settings.theme": "主题",
    "theme.system": "跟随系统",
    "theme.light": "亮色",
    "theme.dark": "暗色",
    "settings.lang": "语言",
    "lang.zh": "简体中文",
    "lang.en": "English",
    "settings.library": "统一库位置",
    "settings.libraryPath": "当前: {path}",
    "settings.change": "更改",
    "settings.restore": "恢复默认",
    "settings.libraryChanged": "统一库位置已更新",
    "settings.libraryFailed": "设置失败",
  },
  en: {
    "app.title": "SkillNexus",
    "app.subtitle": "Manage skills across agents",
    "btn.install": "Install Skill",
    "btn.sync": "Sync All",
    "btn.scan": "Scan",
    "btn.settings": "Settings",
    "theme.toLight": "Switch to light",
    "theme.toDark": "Switch to dark",
    "search.placeholder": "Search skills...",
    "list.empty": "The skill library is empty. Click \"Install Skill\" to add one.",
    "list.noMatch": "No matching skills",
    "status.count": "{n} skills | Library: {path}",
    "status.loadFailed": "Load failed",
    "agent.connected": "connected",
    "agent.notSynced": "not synced",
    "detail.noDesc": "No description",
    "detail.select": "Select a skill on the left to view details",
    "detail.openFolder": "Open folder",
    "detail.skillMd": "SKILL.md",
    "detail.noMd": "(No SKILL.md or failed to read)",
    "scan.title": "Scan Report",
    "scan.unified": "Unified Library",
    "scan.skills": "{n} skills",
    "scan.agents": "Agents detected on this machine",
    "scan.installed": "Installed",
    "scan.kind.link": "Link",
    "scan.kind.direct": "Direct",
    "scan.synced": "Synced {n}/{total}",
    "scan.connected": "Connected",
    "scan.path.unified": "(unified library)",
    "scan.hint": "To add other agents, edit ~/.agents/agents.json then rescan.",
    "scan.close": "Close",
    "toast.installed": "Skill \"{name}\" installed",
    "toast.installFailed": "Install failed",
    "toast.syncDone": "Sync done: {ok} ok, {fail} failed",
    "toast.syncFailed": "Sync failed",
    "settings.title": "Settings",
    "settings.theme": "Theme",
    "theme.system": "System",
    "theme.light": "Light",
    "theme.dark": "Dark",
    "settings.lang": "Language",
    "lang.zh": "简体中文",
    "lang.en": "English",
    "settings.library": "Unified Library Location",
    "settings.libraryPath": "Current: {path}",
    "settings.change": "Change",
    "settings.restore": "Restore default",
    "settings.libraryChanged": "Library location updated",
    "settings.libraryFailed": "Failed to update",
  },
};

export type TFunc = (key: string, vars?: Record<string, string | number>) => string;

export function makeT(lang: Lang): TFunc {
  return (key, vars) => {
    let s = messages[lang][key] ?? messages.zh[key] ?? key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        s = s.replace(`{${k}}`, String(v));
      }
    }
    return s;
  };
}

export function getInitialLang(): Lang {
  const saved = localStorage.getItem("lang");
  return saved === "en" ? "en" : "zh";
}
