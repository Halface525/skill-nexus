import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { AgentBadge } from "./AgentBadge";
import { makeT, getInitialLang, type Lang } from "./i18n";
import {
  IconOrbit,
  IconSearch,
  IconSun,
  IconMoon,
  IconPlus,
  IconRefresh,
  IconScan,
  IconFolder,
  IconX,
  IconSettings,
} from "./icons";
import "./App.css";

interface AgentStatus {
  name: string;
  kind: string;
  ok: boolean;
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
  kind: string;
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
}

interface Toast {
  type: "success" | "error";
  text: string;
}

type ThemeSetting = "system" | "light" | "dark";

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
  const [toast, setToast] = useState<Toast | null>(null);
  const toastTimer = useRef<number | null>(null);

  const t = makeT(lang);
  const eff = effectiveTheme(theme);
  const statusText = loadError
    ? `${t("status.loadFailed")}: ${loadError}`
    : t("status.count", { n: skills.length, path: libraryPath || "~/.agents/skills" });

  // 主题:应用 + 持久化 + 系统主题变化监听
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", eff);
    localStorage.setItem("theme", theme);
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
      .then((s) => setLibraryPath(s.unifiedLibrary))
      .catch(() => {});
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
      showToast("success", t("settings.libraryChanged"));
      const s = await invoke<SettingsView>("get_settings");
      setSettings(s);
      setLibraryPath(s.unifiedLibrary);
      await refresh();
    } catch (e) {
      showToast("error", `${t("settings.libraryFailed")}: ${e}`);
    }
  }

  return (
    <div className="app">
      {/* 顶栏 */}
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
            onClick={() => setTheme(eff === "dark" ? "light" : "dark")}
            title={eff === "dark" ? t("theme.toLight") : t("theme.toDark")}
          >
            {eff === "dark" ? <IconSun size={16} /> : <IconMoon size={16} />}
          </button>
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
                <button
                  className="btn sm"
                  onClick={() => handleOpenDir(selected)}
                >
                  <IconFolder size={14} />
                  {t("detail.openFolder")}
                </button>
              </div>
              <p className="detail-desc">
                {selected.description || t("detail.noDesc")}
              </p>
              <div className="detail-agents">
                {selected.agents.map((a) => (
                  <AgentBadge
                    key={a.name}
                    name={a.name}
                    ok={a.ok}
                    size="lg"
                    label
                    lang={lang}
                  />
                ))}
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

              <div className="scan-subtitle">{t("scan.agents")}</div>

              {scan.agents.map((a) => (
                <div key={a.name} className="scan-card">
                  <div className="scan-card-row">
                    <span className="scan-agent">
                      <AgentBadge name={a.name} ok lang={lang} />
                      {a.name}
                    </span>
                    <div className="pill-wrap">
                      <span className="pill neutral">
                        {a.kind === "junction"
                          ? t("scan.kind.link")
                          : t("scan.kind.direct")}
                      </span>
                      {a.kind === "junction" ? (
                        <span
                          className={`pill ${a.syncedCount >= scan.unifiedCount ? "ok" : "warn"}`}
                        >
                          {t("scan.synced", {
                            n: a.syncedCount,
                            total: scan.unifiedCount,
                          })}
                        </span>
                      ) : (
                        <span className="pill ok">{t("scan.connected")}</span>
                      )}
                    </div>
                  </div>
                  <div className="scan-path">
                    {a.root}
                    {a.kind === "direct" ? t("scan.path.unified") : ""} ·{" "}
                    {t("scan.skills", { n: a.skillCount })}
                  </div>
                </div>
              ))}

              <div className="scan-hint">{t("scan.hint")}</div>
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
