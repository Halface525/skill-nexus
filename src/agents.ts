// ── Agent 品牌信息:颜色 + 缩写 + logo ────────────────
import {
  siAmp,
  siClaude,
  siCline,
  siCursor,
  siDeepseek,
  siGithub,
  siGoogle,
  siGooglegemini,
  siKimi,
  siOpencode,
  siQwen,
  siTrae,
  siWindsurf,
} from "simple-icons";

export interface AgentMeta {
  color: string; // 品牌色(白底上 mark 的颜色 / 无 logo 时徽章底色)
  abbr: string;
  path?: string; // simple-icons 单色 mark(白底 + 品牌色渲染)
  svg?: string; // 完整彩色 logo SVG(作为徽章背景图)
}

// simple-icons 提供单色 mark,在白底上用品牌色渲染
function icon(si: { hex: string; path: string }, abbr: string): AgentMeta {
  return { color: si.hex, abbr, path: si.path };
}

// ── ChatGPT 完整彩色 logo(Codex 用,代表 GPT/OpenAI 系)──
const KNOT =
  "M1107.3 299.1c-197.999 0-373.9 127.3-435.2 315.3L650 743.5v427.9c0 21.4 11 40.4 29.4 51.4l344.5 198.515V833.3h.1v-27.9L1372.7 604c33.715-19.52 70.44-32.857 108.47-39.828L1447.6 450.3C1361 353.5 1237.1 298.5 1107.3 299.1zm0 117.5-.6.6c79.699 0 156.3 27.5 217.6 78.4-2.5 1.2-7.4 4.3-11 6.1L952.8 709.3c-18.4 10.4-29.4 30-29.4 51.4V1248l-155.1-89.4V755.8c-.1-187.099 151.601-338.9 339-339.2z";

const CHATGPT_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2406 2406"><path d="M1 578.4C1 259.5 259.5 1 578.4 1h1249.1c319 0 577.5 258.5 577.5 577.4V2406H578.4C259.5 2406 1 2147.5 1 1828.6V578.4z" fill="#74aa9c"/>${[0, 60, 120, 180, 240, 300]
  .map(
    (r) =>
      `<path d="${KNOT}"${r ? ` transform="rotate(${r} 1203 1203)"` : ""} fill="#fff"/>`,
  )
  .join("")}</svg>`;

export const AGENT_META: Record<string, AgentMeta> = {
  "DeepSeek Harness": icon(siDeepseek, "DS"),
  Codex: { color: "#10a37f", abbr: "Cx", svg: CHATGPT_SVG },
  Cursor: icon(siCursor, "Cu"),
  Antigravity: icon(siGoogle, "An"),
  Aider: { color: "#2563eb", abbr: "Ai" },
  Windsurf: icon(siWindsurf, "Wi"),
  Trae: icon(siTrae, "Tr"),
  "GitHub Copilot": icon(siGithub, "GC"),
  Devin: { color: "#f97316", abbr: "De" },
  Amp: icon(siAmp, "Am"),
  Claude: icon(siClaude, "Cl"),
  Cline: icon(siCline, "Cn"),
  "Roo Code": { color: "#14b8a6", abbr: "Ro" },
  Gemini: icon(siGooglegemini, "Ge"),
  OpenCode: icon(siOpencode, "Oc"),
  Goose: { color: "#64748b", abbr: "Go" },
  Kiro: { color: "#ec4899", abbr: "Ki" },
  WorkBuddy: { color: "#334155", abbr: "Wb" },
  "Qwen Code": icon(siQwen, "Qw"),
  "Kilo Code": { color: "#10b981", abbr: "Kl" },
  OpenHands: { color: "#0d9488", abbr: "Oh" },
  iFlow: { color: "#0f62fe", abbr: "iF" },
  Kimi: icon(siKimi, "Km"),
  Grok: { color: "#111827", abbr: "Gr" },
};

export function agentMeta(name: string): AgentMeta {
  return (
    AGENT_META[name] ?? {
      color: "#64748b",
      abbr: name.replace(/[^a-zA-Z]/g, "").slice(0, 2).toUpperCase() || "?",
    }
  );
}
