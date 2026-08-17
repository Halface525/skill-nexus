import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { AgentBadge } from "./AgentBadge";
import { makeT, getInitialLang, type Lang } from "./i18n";
import {
  IconOrbit,
  IconSearch,
  IconPlus,
  IconRefresh,
  IconScan,
  IconFolder,
  IconX,
  IconSettings,
  IconTrash,
  IconAlert,
  IconMinimize,
  IconMaximize,
} from "./icons";
import "./App.css";

interface AgentStatus {
  name: string;
  kind: string;
  ok: boolean;
  excluded: boolean;
  controllable: boolean;
}

interface Skill {
  name: string;
  description: string;
  dir: string;
  hasSkillMd: boolean;
  agents: AgentStatus[];
}

interface SyncResult {
  ok: number;
  fail: number;
  total: number;
}

interface AgentScan {
  name: string;
  installed: boolean;
  enabled: boolean;
  kind: string;
  controllable: boolean;
  root: string;
  skillCount: number;
  syncedCount: number;
}

interface ScanInfo {
  unifiedRoot: string;
  unifiedCount: number;
  agents: AgentScan[];
}

interface SettingsView {
  unifiedLibrary: string;
  defaultLibrary: string;
  librarySetup: boolean;
  suggestedLibrary: string;
}

interface Toast {
  type: "success" | "error";
  text: string;
}

type ThemeSetting = "system" | "light" | "dark";

type UpdateState = "idle" | "checking" | "latest" | "available" | "error";

function compareVersions(a: string, b: string): number {
  const pa = a.split(".").map((n) => parseInt(n, 10) || 0);
  const pb = b.split(".").map((n) => parseInt(n, 10) || 0);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (x !== y) return x > y ? 1 : -1;
  }
  return 0;
}

function getInitialTheme(): ThemeSetting {
  const saved = localStorage.getItem("theme");
  return saved === "light" || saved === "dark" || saved === "system"
    ? saved
    : "system";
}

function effectiveTheme(t: ThemeSetting): "light" | "dark" {
  if (t === "light" || t === "dark") return t;
  return window.matchMedia("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

function Switch({
  checked,
  onChange,
  title,
}: {
  checked: boolean;
  onChange: () => void;
  title?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      className={`switch ${checked ? "on" : ""}`}
      onClick={onChange}
      title={title}
    >
      <span className="switch-knob" />
    </button>
  );
}

function App() {
  const [theme, setTheme] = useState<ThemeSetting>(getInitialTheme);
  const [lang, setLang] = useState<Lang>(getInitialLang);
  const [skills, setSkills] = useState<Skill[]>([]);
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<Skill | null>(null);
  const [skillMd, setSkillMd] = useState("");
  const [scan, setScan] = useState<ScanInfo | null>(null);
  const [loadError, setLoadError] = useState("");
  const [libraryPath, setLibraryPath] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settings, setSettings] = useState<SettingsView | null>(null);
  const [uninstallTarget, setUninstallTarget] = useState<Skill | null>(null);
  const [appVersion, setAppVersion] = useState("");
  const [updateState, setUpdateState] = useState<UpdateState>("idle");
  const [latestVersion, setLatestVersion] = useState("");
  const [setupOpen, setSetupOpen] = useState(false);
  const [suggestedLibrary, setSuggestedLibrary] = useState("");
  const [agentFormOpen, setAgentFormOpen] = useState(false);
  const [agentForm, setAgentForm] = useState({
    name: "",
    kind: "junction" as "direct" | "junction",
    dir: "",
    binary: "",
    homeDir: "",
    manual: false,
  });
  const [toast, setToast] = useState<Toast | null>(null);
  const toastTimer = useRef<number | null>(null);
  const win = getCurrentWindow();

  const t = makeT(lang);
  const eff = effectiveTheme(theme);
  const statusText = loadError
    ? `${t("status.loadFailed")}: ${loadError}`
    : t("status.count", { n: skills.length, path: libraryPath || "~/.agents/skills" });

  // 主题:应用 + 持久化 + 系统主题变化监听 + 原生标题栏跟随
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", eff);
    localStorage.setItem("theme", theme);
    // 让 Windows 原生标题栏(最小化/最大化/关闭那一条)跟随应用主题
    // "system" → null(跟随系统)
    const win = getCurrentWindow();
    (theme === "system"
      ? win.setTheme(null)
      : win.setTheme(theme as "light" | "dark")
    ).catch(() => {});
  }, [eff, theme]);
  useEffect(() => {
    if (theme !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: light)");
    const onChange = () =>
      document.documentElement.setAttribute("data-theme", effectiveTheme("system"));
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [theme]);

  // 语言持久化
  useEffect(() => {
    localStorage.setItem("lang", lang);
  }, [lang]);

  function showToast(type: Toast["type"], text: string) {
    setToast({ type, text });
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), 3200);
  }

  async function refresh() {
    try {
      const list = await invoke<Skill[]>("load_skills");
      setSkills(list);
      setLoadError("");
      setSelected((prev) =>
        prev ? list.find((s) => s.name === prev.name) ?? null : null,
      );
    } catch (e) {
      setLoadError(String(e));
    }
  }

  useEffect(() => {
    invoke<SettingsView>("get_settings")
      .then((s) => {
        setLibraryPath(s.unifiedLibrary);
        setSuggestedLibrary(s.suggestedLibrary);
        // 首次运行:引导选择统一库位置
        if (!s.librarySetup) setSetupOpen(true);
      })
      .catch(() => {});
    getVersion().then(setAppVersion).catch(() => {});
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!selected) {
      setSkillMd("");
      return;
    }
    invoke<string>("read_skill_md", { dir: selected.dir, lang })
      .then(setSkillMd)
      .catch(() => setSkillMd(""));
  }, [selected, lang]);

  const q = search.trim().toLowerCase();
  const filtered = skills.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.description.toLowerCase().includes(q),
  );

  async function handleInstall() {
    const dir = await open({
      directory: true,
      title: t("btn.install"),
    });
    if (!dir) return;
    try {
      const name = await invoke<string>("install_skill", { src: dir, lang });
      showToast("success", t("toast.installed", { name }));
      await refresh();
    } catch (e) {
      showToast("error", `${t("toast.installFailed")}: ${e}`);
    }
  }

  async function handleSync() {
    try {
      const r = await invoke<SyncResult>("sync_all");
      showToast("success", t("toast.syncDone", { ok: r.ok, fail: r.fail }));
      await refresh();
    } catch (e) {
      showToast("error", `${t("toast.syncFailed")}: ${e}`);
    }
  }

  async function handleScan() {
    setScan(await invoke<ScanInfo>("scan_info"));
  }

  async function handleOpenDir(skill: Skill) {
    try {
      await invoke("open_dir", { path: skill.dir });
    } catch (e) {
      console.error(e);
    }
  }

  async function handleUninstall() {
    if (!uninstallTarget) return;
    const name = uninstallTarget.name;
    try {
      await invoke("uninstall_skill", { name, lang });
      showToast("success", t("toast.uninstalled", { name }));
      setUninstallTarget(null);
      setSelected(null);
      await refresh();
    } catch (e) {
      showToast("error", `${t("toast.uninstallFailed")}: ${e}`);
      setUninstallTarget(null);
    }
  }

  async function openSettings() {
    try {
      const s = await invoke<SettingsView>("get_settings");
      setSettings(s);
      setLibraryPath(s.unifiedLibrary);
    } catch {
      /* ignore */
    }
    setSettingsOpen(true);
  }

  async function handleChangeLibrary() {
    const dir = await open({ directory: true, title: t("settings.library") });
    if (!dir) return;
    try {
      await invoke("set_unified_library", { path: dir });
      await invoke<SyncResult>("sync_all"); // 迁移后立即同步链接
      showToast("success", t("settings.libraryChanged"));
      const s = await invoke<SettingsView>("get_settings");
      setSettings(s);
      setLibraryPath(s.unifiedLibrary);
      await refresh();
    } catch (e) {
      showToast("error", `${t("settings.libraryFailed")}: ${e}`);
    }
  }

  async function handleRestoreLibrary() {
    if (!settings) return;
    try {
      await invoke("set_unified_library", { path: settings.defaultLibrary });
      await invoke<SyncResult>("sync_all");
      showToast("success", t("settings.libraryChanged"));
      const s = await invoke<SettingsView>("get_settings");
      setSettings(s);
      setLibraryPath(s.unifiedLibrary);
      await refresh();
    } catch (e) {
      showToast("error", `${t("settings.libraryFailed")}: ${e}`);
    }
  }

  // 首次运行:把统一库迁移到指定位置(迁移 + 同步)
  async function handleSetupLibrary(target: string) {
    try {
      await invoke("set_unified_library", { path: target });
      await invoke<SyncResult>("sync_all");
      const s = await invoke<SettingsView>("get_settings");
      setLibraryPath(s.unifiedLibrary);
      setSetupOpen(false);
      await refresh();
      showToast("success", t("setup.done"));
    } catch (e) {
      showToast("error", `${t("setup.failed")}: ${e}`);
    }
  }

  async function handleSetupChooseOther() {
    const dir = await open({ directory: true, title: t("setup.title") });
    if (!dir) return;
    await handleSetupLibrary(dir);
  }

  async function handleCheckUpdate() {
    setUpdateState("checking");
    try {
      const res = await fetch(
        "https://api.github.com/repos/Halface525/skill-nexus/releases/latest",
      );
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = (await res.json()) as { tag_name?: string };
      const latest = String(data.tag_name ?? "").replace(/^v/i, "");
      const cur = appVersion || "0";
      setLatestVersion(latest);
      setUpdateState(
        latest && compareVersions(latest, cur) > 0 ? "available" : "latest",
      );
    } catch {
      setUpdateState("error");
    }
  }

  async function handleToggleAgent(a: AgentScan) {
    const next = !a.enabled;
    try {
      await invoke("set_agent_enabled", { name: a.name, enabled: next, lang });
      setScan(await invoke<ScanInfo>("scan_info"));
      await refresh();
      showToast(
        "success",
        next
          ? t("toast.agentEnabled", { name: a.name })
          : t("toast.agentDisabled", { name: a.name }),
      );
    } catch (e) {
      showToast("error", `${e}`);
    }
  }

  async function handleToggleSkillAgent(skill: Skill, a: AgentStatus) {
    const wantEnabled = a.excluded; // 当前被排除 → 本次改为启用
    try {
      await invoke("set_skill_agent", {
        skill: skill.name,
        agent: a.name,
        enabled: wantEnabled,
        lang,
      });
      await refresh();
      showToast(
        "success",
        wantEnabled
          ? t("toast.skillAgentOn", { skill: skill.name, agent: a.name })
          : t("toast.skillAgentOff", { skill: skill.name, agent: a.name }),
      );
    } catch (e) {
      showToast("error", `${e}`);
    }
  }

  async function handleAddAgent() {
    try {
      const name = await invoke<string>("add_agent", {
        agent: {
          name: agentForm.name,
          kind: agentForm.kind,
          dir: agentForm.dir.trim() || null,
          binary: agentForm.binary.trim() || null,
          homeDir: agentForm.homeDir.trim() || null,
          appdata: [],
          manual: agentForm.manual,
        },
        lang,
      });
      setAgentFormOpen(false);
      setAgentForm({
        name: "",
        kind: "junction",
        dir: "",
        binary: "",
        homeDir: "",
        manual: false,
      });
      setScan(await invoke<ScanInfo>("scan_info"));
      await refresh();
      showToast("success", t("toast.agentAdded", { name }));
    } catch (e) {
      showToast("error", `${t("toast.agentAddFailed")}: ${e}`);
    }
  }

  return (
    <div className="app">
      {/* 自绘标题栏:可拖拽,双击最大化/还原,右侧窗口控制 */}
      <div
        className="titlebar"
        data-tauri-drag-region
        onDoubleClick={() => win.toggleMaximize()}
      >
        <div className="titlebar-title" data-tauri-drag-region>
          <IconOrbit size={15} />
          <span>{t("app.title")}</span>
        </div>
        <div className="window-controls" onDoubleClick={(e) => e.stopPropagation()}>
          <button
            className="wc-btn"
            title={t("win.minimize")}
            onClick={() => win.minimize()}
          >
            <IconMinimize size={14} />
          </button>
          <button
            className="wc-btn"
            title={t("win.maximize")}
            onClick={() => win.toggleMaximize()}
          >
            <IconMaximize size={13} />
          </button>
          <button
            className="wc-btn wc-close"
            title={t("win.close")}
            onClick={() => win.close()}
          >
            <IconX size={14} />
          </button>
        </div>
      </div>

      {/* 顶栏:品牌 + 操作 */}
      <header className="header">
        <div className="brand">
          <div className="logo">
            <IconOrbit size={26} />
          </div>
          <div>
            <h1>{t("app.title")}</h1>
            <p>{t("app.subtitle")}</p>
          </div>
        </div>
        <div className="actions">
          <button
            className="btn icon"
            onClick={openSettings}
            title={t("btn.settings")}
          >
            <IconSettings size={16} />
          </button>
          <button className="btn primary" onClick={handleInstall}>
            <IconPlus size={15} />
            {t("btn.install")}
          </button>
          <button className="btn" onClick={handleSync}>
            <IconRefresh size={15} />
            {t("btn.sync")}
          </button>
          <button className="btn" onClick={handleScan}>
            <IconScan size={15} />
            {t("btn.scan")}
          </button>
        </div>
      </header>

      {/* 搜索 */}
      <div className="searchbar">
        <div className="search-wrap">
          <span className="search-icon">
            <IconSearch size={14} />
          </span>
          <input
            className="search"
            placeholder={t("search.placeholder")}
            value={search}
            onChange={(e) => setSearch(e.currentTarget.value)}
          />
        </div>
      </div>

      {/* 主体:列表 + 详情 */}
      <main className="body">
        <div className="list">
          {filtered.length === 0 ? (
            <div className="empty">
              {skills.length === 0 ? t("list.empty") : t("list.noMatch")}
            </div>
          ) : (
            filtered.map((s) => (
              <div
                key={s.name}
                className={`item ${selected?.name === s.name ? "active" : ""}`}
                onClick={() => setSelected(s)}
                onDoubleClick={() => handleOpenDir(s)}
              >
                <div className="item-row">
                  <span className={`dot ${s.hasSkillMd ? "ok" : ""}`} />
                  <span className="item-name">{s.name}</span>
                </div>
                {s.description && (
                  <div className="item-desc">{s.description}</div>
                )}
                <div className="agent-dots">
                  {s.agents.map((a) => (
                    <AgentBadge key={a.name} name={a.name} ok={a.ok} lang={lang} />
                  ))}
                </div>
              </div>
            ))
          )}
        </div>

        <div className="detail">
          {selected ? (
            <>
              <div className="detail-header">
                <h2>{selected.name}</h2>
                <div className="detail-actions">
                  <button
                    className="btn sm ghost-danger"
                    onClick={() => setUninstallTarget(selected)}
                    title={t("detail.uninstall")}
                  >
                    <IconTrash size={14} />
                    {t("detail.uninstall")}
                  </button>
                  <button
                    className="btn sm"
                    onClick={() => handleOpenDir(selected)}
                  >
                    <IconFolder size={14} />
                    {t("detail.openFolder")}
                  </button>
                </div>
              </div>
              <div className="detail-desc">
                {selected.description || t("detail.noDesc")}
              </div>
              <div className="detail-agents">
                <div className="md-title">{t("detail.agents")}</div>
                {selected.agents.length === 0 ? (
                  <p className="md-empty">{t("detail.noAgents")}</p>
                ) : (
                  selected.agents.map((a) => (
                    <div key={a.name} className="skill-agent-row">
                      <AgentBadge
                        name={a.name}
                        ok={a.ok}
                        size="lg"
                        label
                        lang={lang}
                      />
                      {a.controllable ? (
                        <Switch
                          checked={!a.excluded}
                          onChange={() => handleToggleSkillAgent(selected, a)}
                        />
                      ) : (
                        <span className="skill-agent-note">
                          {t("agent.directRead")}
                        </span>
                      )}
                    </div>
                  ))
                )}
              </div>
              <div className="md-title">{t("detail.skillMd")}</div>
              <div className="markdown">
                {skillMd ? (
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>
                    {skillMd}
                  </ReactMarkdown>
                ) : (
                  <p className="md-empty">{t("detail.noMd")}</p>
                )}
              </div>
            </>
          ) : (
            <div className="detail-empty">{t("detail.select")}</div>
          )}
        </div>
      </main>

      {/* 状态栏 */}
      <footer className="footer">{statusText}</footer>

      {/* 扫描面板 */}
      {scan && (
        <div className="overlay" onClick={() => setScan(null)}>
          <div className="overlay-box" onClick={(e) => e.stopPropagation()}>
            <div className="overlay-header">
              <span className="overlay-title">{t("scan.title")}</span>
              <button
                className="btn icon"
                onClick={() => setScan(null)}
                title={t("scan.close")}
              >
                <IconX size={16} />
              </button>
            </div>
            <div className="scan-body">
              <div className="scan-card">
                <div className="scan-card-row">
                  <span className="scan-label">{t("scan.unified")}</span>
                  <span className="pill neutral">
                    {t("scan.skills", { n: scan.unifiedCount })}
                  </span>
                </div>
                <div className="scan-path">{scan.unifiedRoot}</div>
              </div>

              <div className="scan-subtitle">
                <span>{t("scan.agents")}</span>
                <button className="btn sm" onClick={() => setAgentFormOpen(true)}>
                  <IconPlus size={13} />
                  {t("agent.add")}
                </button>
              </div>

              {scan.agents.map((a) => (
                <div key={a.name} className="scan-card">
                  <div className="scan-card-row">
                    <span className="scan-agent">
                      <AgentBadge
                        name={a.name}
                        ok={
                          !a.controllable
                            ? true
                            : a.enabled &&
                              a.syncedCount >= scan.unifiedCount
                        }
                        lang={lang}
                      />
                      {a.name}
                      {!a.enabled && (
                        <span className="pill warn">{t("agent.disabled")}</span>
                      )}
                    </span>
                    <div className="pill-wrap">
                      {a.controllable ? (
                        <>
                          <Switch
                            checked={a.enabled}
                            onChange={() => handleToggleAgent(a)}
                          />
                          <span className="pill neutral">
                            {t("scan.kind.link")}
                          </span>
                          {a.enabled && (
                            <span
                              className={`pill ${
                                a.syncedCount >= scan.unifiedCount
                                  ? "ok"
                                  : "warn"
                              }`}
                            >
                              {t("scan.synced", {
                                n: a.syncedCount,
                                total: scan.unifiedCount,
                              })}
                            </span>
                          )}
                        </>
                      ) : (
                        <span className="pill ok" title={t("agent.directNote")}>
                          {t("scan.connected")}
                        </span>
                      )}
                    </div>
                  </div>
                  <div className="scan-path">
                    {a.root}
                    {a.root === scan.unifiedRoot ? t("scan.path.unified") : ""}{" "}
                    · {t("scan.skills", { n: a.skillCount })}
                  </div>
                </div>
              ))}

            </div>
          </div>
        </div>
      )}

      {/* 设置面板 */}
      {settingsOpen && (
        <div className="overlay" onClick={() => setSettingsOpen(false)}>
          <div
            className="overlay-box settings"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="overlay-header">
              <span className="overlay-title">{t("settings.title")}</span>
              <button
                className="btn icon"
                onClick={() => setSettingsOpen(false)}
                title={t("scan.close")}
              >
                <IconX size={16} />
              </button>
            </div>
            <div className="settings-body">
              <div className="settings-section">
                <div className="settings-label">{t("settings.theme")}</div>
                <div className="seg">
                  {(["system", "light", "dark"] as const).map((v) => (
                    <button
                      key={v}
                      className={`seg-btn ${theme === v ? "active" : ""}`}
                      onClick={() => setTheme(v)}
                    >
                      {t(`theme.${v}`)}
                    </button>
                  ))}
                </div>
              </div>

              <div className="settings-section">
                <div className="settings-label">{t("settings.lang")}</div>
                <div className="seg">
                  <button
                    className={`seg-btn ${lang === "zh" ? "active" : ""}`}
                    onClick={() => setLang("zh")}
                  >
                    {t("lang.zh")}
                  </button>
                  <button
                    className={`seg-btn ${lang === "en" ? "active" : ""}`}
                    onClick={() => setLang("en")}
                  >
                    {t("lang.en")}
                  </button>
                </div>
              </div>

              <div className="settings-section">
                <div className="settings-label">{t("settings.library")}</div>
                {settings && (
                  <div className="settings-path">
                    {t("settings.libraryPath", { path: settings.unifiedLibrary })}
                  </div>
                )}
                <div className="settings-actions">
                  <button className="btn sm" onClick={handleChangeLibrary}>
                    <IconFolder size={13} />
                    {t("settings.change")}
                  </button>
                  <button className="btn sm" onClick={handleRestoreLibrary}>
                    {t("settings.restore")}
                  </button>
                </div>
              </div>

              <div className="settings-section">
                <div className="settings-label">{t("settings.agents")}</div>
                <div className="settings-actions">
                  <button
                    className="btn sm"
                    onClick={() => {
                      setSettingsOpen(false);
                      setAgentFormOpen(true);
                    }}
                  >
                    <IconPlus size={13} />
                    {t("agent.add")}
                  </button>
                </div>
              </div>

              <div className="settings-section">
                <div className="settings-label">{t("settings.about")}</div>
                <div className="about-card">
                  <div className="about-row">
                    <div className="about-logo">
                      <IconOrbit size={22} />
                    </div>
                    <div className="about-info">
                      <div className="about-name">SkillNexus</div>
                      <div className="about-meta">
                        {t("about.version", { v: appVersion || "…" })}
                      </div>
                      <div className="about-meta">{t("about.author")}</div>
                    </div>
                  </div>
                  <div className="about-actions">
                    <button
                      className="btn sm"
                      onClick={() =>
                        openUrl("https://github.com/Halface525/skill-nexus").catch(
                          () => {},
                        )
                      }
                    >
                      {t("about.github")}
                    </button>
                    <button className="btn sm" onClick={handleCheckUpdate}>
                      {t("about.checkUpdate")}
                    </button>
                  </div>
                  {updateState === "checking" && (
                    <div className="about-update">{t("about.checking")}</div>
                  )}
                  {updateState === "latest" && (
                    <div className="about-update ok">✓ {t("about.latest")}</div>
                  )}
                  {updateState === "available" && (
                    <div className="about-update warn">
                      {t("about.updateAvailable", { v: latestVersion })}
                      <button
                        className="btn sm"
                        onClick={() =>
                          openUrl(
                            "https://github.com/Halface525/skill-nexus/releases",
                          ).catch(() => {})
                        }
                      >
                        {t("about.download")}
                      </button>
                    </div>
                  )}
                  {updateState === "error" && (
                    <div className="about-update err">
                      {t("about.checkFailed")}
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 卸载确认弹窗 */}
      {uninstallTarget && (
        <div className="overlay" onClick={() => setUninstallTarget(null)}>
          <div
            className="overlay-box confirm"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="confirm-icon">
              <IconAlert size={24} />
            </div>
            <div className="confirm-title">{t("uninstall.title")}</div>
            <div className="confirm-text">
              {t("uninstall.confirm", { name: uninstallTarget.name })}
            </div>
            <div className="confirm-actions">
              <button className="btn" onClick={() => setUninstallTarget(null)}>
                {t("uninstall.cancel")}
              </button>
              <button className="btn danger" onClick={handleUninstall}>
                <IconTrash size={14} />
                {t("uninstall.confirmBtn")}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 添加自定义 Agent 表单 */}
      {agentFormOpen && (
        <div className="overlay" onClick={() => setAgentFormOpen(false)}>
          <div
            className="overlay-box form"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="overlay-header">
              <span className="overlay-title">{t("agent.add")}</span>
              <button
                className="btn icon"
                onClick={() => setAgentFormOpen(false)}
                title={t("scan.close")}
              >
                <IconX size={16} />
              </button>
            </div>
            <div className="form-body">
              <div className="form-field">
                <label>{t("agent.form.name")}</label>
                <input
                  value={agentForm.name}
                  placeholder={t("agent.form.nameHint")}
                  autoFocus
                  onChange={(e) =>
                    setAgentForm({ ...agentForm, name: e.currentTarget.value })
                  }
                />
              </div>
              <div className="form-field">
                <label>{t("agent.form.kind")}</label>
                <div className="seg">
                  <button
                    className={`seg-btn ${
                      agentForm.kind === "direct" ? "active" : ""
                    }`}
                    onClick={() =>
                      setAgentForm({ ...agentForm, kind: "direct" })
                    }
                  >
                    {t("agent.form.kindDirect")}
                  </button>
                  <button
                    className={`seg-btn ${
                      agentForm.kind === "junction" ? "active" : ""
                    }`}
                    onClick={() =>
                      setAgentForm({ ...agentForm, kind: "junction" })
                    }
                  >
                    {t("agent.form.kindJunction")}
                  </button>
                </div>
              </div>
              {agentForm.kind === "junction" && (
                <div className="form-field">
                  <label>{t("agent.form.dir")}</label>
                  <input
                    value={agentForm.dir}
                    placeholder={t("agent.form.dirHint")}
                    onChange={(e) =>
                      setAgentForm({ ...agentForm, dir: e.currentTarget.value })
                    }
                  />
                </div>
              )}
              <div className="form-field">
                <label>{t("agent.form.binary")}</label>
                <input
                  value={agentForm.binary}
                  placeholder={t("agent.form.binaryHint")}
                  onChange={(e) =>
                    setAgentForm({
                      ...agentForm,
                      binary: e.currentTarget.value,
                    })
                  }
                />
              </div>
              <div className="form-field">
                <label>{t("agent.form.homeDir")}</label>
                <input
                  value={agentForm.homeDir}
                  placeholder={t("agent.form.homeDirHint")}
                  onChange={(e) =>
                    setAgentForm({
                      ...agentForm,
                      homeDir: e.currentTarget.value,
                    })
                  }
                />
              </div>
              <label className="form-check">
                <input
                  type="checkbox"
                  checked={agentForm.manual}
                  onChange={(e) =>
                    setAgentForm({
                      ...agentForm,
                      manual: e.currentTarget.checked,
                    })
                  }
                />
                <span>{t("agent.form.manual")}</span>
              </label>
              <div className="form-actions">
                <button
                  className="btn"
                  onClick={() => setAgentFormOpen(false)}
                >
                  {t("agent.form.cancel")}
                </button>
                <button className="btn primary" onClick={handleAddAgent}>
                  <IconPlus size={14} />
                  {t("agent.form.save")}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 首次运行:选择统一库位置 */}
      {setupOpen && (
        <div className="overlay">
          <div className="overlay-box setup" onClick={(e) => e.stopPropagation()}>
            <div className="setup-icon">
              <IconOrbit size={26} />
            </div>
            <div className="setup-title">{t("setup.title")}</div>
            <div className="setup-desc">{t("setup.desc")}</div>
            <div className="setup-path">
              <span className="setup-path-label">{t("setup.suggested")}</span>
              <code>{suggestedLibrary || libraryPath}</code>
            </div>
            <div className="setup-actions">
              <button
                className="btn primary"
                onClick={() =>
                  handleSetupLibrary(suggestedLibrary || libraryPath)
                }
              >
                {t("setup.useSuggested")}
              </button>
              <button className="btn" onClick={handleSetupChooseOther}>
                {t("setup.chooseOther")}
              </button>
              <button
                className="btn ghost"
                onClick={() => handleSetupLibrary(libraryPath)}
              >
                {t("setup.keepDefault")}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 轻提示 toast */}
      {toast && (
        <div className="toast-wrap">
          <div className={`toast ${toast.type}`}>
            <span className={`toast-dot ${toast.type}`} />
            {toast.text}
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
