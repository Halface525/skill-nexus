import { agentMeta } from "./agents";
import { makeT, type Lang } from "./i18n";

function svgDataUri(svg: string): string {
  return `url("data:image/svg+xml,${encodeURIComponent(svg)}")`;
}

interface Props {
  name: string;
  ok: boolean;
  size?: "sm" | "lg";
  label?: boolean; // 是否显示名字文本(详情面板用)
  lang: Lang;
}

// 徽章:完整彩色 logo → 白底品牌色 mark → 品牌色字母缩写;角落状态点(绿=已接入,红=未同步)
export function AgentBadge({ name, ok, size = "sm", label = false, lang }: Props) {
  const meta = agentMeta(name);
  const t = makeT(lang);
  const badgeStyle = meta.svg
    ? { backgroundImage: svgDataUri(meta.svg) }
    : meta.path
      ? { backgroundColor: "#ffffff" }
      : { backgroundColor: meta.color };
  return (
    <span
      className={`agent-badge-item ${size}`}
      title={`${name}${ok ? ` · ${t("agent.connected")}` : ` · ${t("agent.notSynced")}`}`}
    >
      <span className="agent-badge" style={badgeStyle}>
        {meta.svg ? null : meta.path ? (
          <svg
            viewBox="0 0 24 24"
            className="agent-logo"
            fill={meta.color}
            aria-hidden="true"
          >
            <path d={meta.path} />
          </svg>
        ) : (
          meta.abbr
        )}
      </span>
      <span className={`agent-status ${ok ? "ok" : "no"}`} />
      {label && (
        <span className={`agent-label ${ok ? "on" : "off"}`}>{name}</span>
      )}
    </span>
  );
}
