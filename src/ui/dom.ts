export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Record<string, string> = {},
  children: Array<HTMLElement | string> = []
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, v);
  for (const child of children) node.append(child);
  return node;
}

export function setStatus(target: HTMLElement, ok: boolean, message: string) {
  target.className = ok ? "muted ok" : "muted err";
  target.textContent = message;
}

export function nonEmpty(v: string | null | undefined): string | null {
  const t = (v ?? "").trim();
  return t ? t : null;
}

export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value = value / 1024;
    unitIndex++;
  }
  if (unitIndex === 0) return String(bytes) + " " + units[unitIndex];
  return value.toFixed(2) + " " + units[unitIndex];
}