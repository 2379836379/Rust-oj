import type { AppServices, IconName } from "../app";
import { el, nonEmpty } from "../dom";
import type {
  AiChatMessage,
  FavoriteFolderInfo,
  ProblemPageInfo,
  ResultPageInfo,
  SubmitLanguageOption,
  SubmitPageInfo,
  SubmitResponse,
  JudgeResponse,
} from "./types";

type ProblemInit = {
  info: ProblemPageInfo;
  contestTitle: string;
};

type CodeMode = "python" | "plain";

function iconButton(
  services: AppServices,
  id: string,
  title: string,
  icon: IconName,
  darkIcon?: IconName
) {
  const btn = el("button", { id, class: "icon-btn", title }, []);
  const img = document.createElement("img");
  img.alt = title;
  services.registerIcon(img, icon, darkIcon);
  btn.append(img);
  return btn;
}

function pythonHighlightHtml(code: string): string {
  const kw = new Set([
    "False",
    "None",
    "True",
    "and",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "class",
    "continue",
    "def",
    "del",
    "elif",
    "else",
    "except",
    "finally",
    "for",
    "from",
    "global",
    "if",
    "import",
    "in",
    "is",
    "lambda",
    "nonlocal",
    "not",
    "or",
    "pass",
    "raise",
    "return",
    "try",
    "while",
    "with",
    "yield",
  ]);

  const esc = (s: string) =>
    s
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;");

  const out: string[] = [];
  let i = 0;
  let mode: "code" | "comment" | "str1" | "str2" | "tri1" | "tri2" = "code";
  const push = (cls: string | null, text: string) => {
    const t = esc(text);
    out.push(cls ? `<span class="${cls}">${t}</span>` : t);
  };

  while (i < code.length) {
    const ch = code[i] ?? "";

    if (mode === "comment") {
      const j = code.indexOf("\\n", i);
      if (j === -1) {
        push("com", code.slice(i));
        break;
      }
      push("com", code.slice(i, j));
      push(null, "\\n");
      i = j + 1;
      mode = "code";
      continue;
    }

    if (mode === "str1" || mode === "str2" || mode === "tri1" || mode === "tri2") {
      const quote = mode === "str1" || mode === "tri1" ? "'" : '"';
      if (mode === "tri1" || mode === "tri2") {
        const tri = quote + quote + quote;
        if (code.startsWith(tri, i)) {
          push("str", tri);
          i += 3;
          mode = "code";
          continue;
        }
        push("str", ch);
        i += 1;
        continue;
      }

      if (ch === "\\") {
        const next = i + 1 < code.length ? code[i + 1] : "";
        push("str", "\\" + next);
        i += 2;
        continue;
      }

      push("str", ch);
      i += 1;
      if (ch === quote) mode = "code";
      continue;
    }

    // code
    if (ch === "#") {
      mode = "comment";
      continue;
    }

    if (code.startsWith("'''", i)) {
      push("str", "'''");
      i += 3;
      mode = "tri1";
      continue;
    }
    if (code.startsWith('"""', i)) {
      push("str", '"""');
      i += 3;
      mode = "tri2";
      continue;
    }

    if (ch === "'") {
      push("str", "'");
      i += 1;
      mode = "str1";
      continue;
    }
    if (ch === '"') {
      push("str", '"');
      i += 1;
      mode = "str2";
      continue;
    }

    if (/[0-9]/.test(ch)) {
      let j = i + 1;
      while (j < code.length && /[0-9_\.]/.test(code[j] ?? "")) j += 1;
      push("num", code.slice(i, j));
      i = j;
      continue;
    }

    if (/[A-Za-z_]/.test(ch)) {
      let j = i + 1;
      while (j < code.length && /[A-Za-z0-9_]/.test(code[j] ?? "")) j += 1;
      const w = code.slice(i, j);
      push(kw.has(w) ? "kw" : null, w);
      i = j;
      continue;
    }

    push(null, ch);
    i += 1;
  }

  return out.join("");
}

function makeEditor(initial: string) {
  const lines = el("pre", { class: "code-lines" }, [""]);
  const hl = el("pre", { class: "code-hl" }, [""]);
  const textarea = el("textarea", { class: "code-area", placeholder: "Write your source code here." }, []) as HTMLTextAreaElement;
  textarea.value = initial;

  const areaWrap = el("div", { class: "code-area-wrap" }, [hl, textarea]);
  const root = el("div", { id: "problemCodeEdit", class: "code-editor-wrap" }, [
    el("div", { class: "code-editor" }, [lines, areaWrap]),
  ]);

  let mode: CodeMode = "python";

  const sync = () => {
    const text = textarea.value;
    const count = Math.max(1, text.split("\\n").length);
    lines.textContent = Array.from({ length: count }, (_, i) => String(i + 1)).join("\\n");
    hl.innerHTML = mode === "python" ? pythonHighlightHtml(text) : "";
    areaWrap.className = mode === "python" ? "code-area-wrap highlight" : "code-area-wrap";
  };

  const setMode = (m: CodeMode) => {
    mode = m;
    sync();
  };

  sync();

  textarea.addEventListener("input", () => sync());
  textarea.addEventListener("scroll", () => {
    lines.scrollTop = textarea.scrollTop;
    hl.scrollTop = textarea.scrollTop;
    hl.scrollLeft = textarea.scrollLeft;
  });

  textarea.addEventListener("keydown", (ev) => {
    const e = ev as KeyboardEvent;
    if (e.key === "Tab") {
      e.preventDefault();
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      textarea.setRangeText("    ", start, end, "end");
      sync();
    }
  });

  return { root, textarea, sync, setMode };
}

function escapeHtml(s: string) {
  return s.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

function translationKey(problemUrl: string, field: "description" | "input" | "output" | "hint") {
  return `tr:${field}:${problemUrl}`;
}

function draftKey(problemUrl: string) {
  return `draft:${problemUrl}`;
}

function langKey(problemUrl: string) {
  return `lang:${problemUrl}`;
}

function parseJsonObject(text: string): any {
  const trimmed = String(text ?? "").trim();
  const start = trimmed.indexOf("{");
  const end = trimmed.lastIndexOf("}");
  if (start >= 0 && end > start) {
    return JSON.parse(trimmed.slice(start, end + 1));
  }
  return JSON.parse(trimmed);
}

function renderProblemDetailHtml(
  info: ProblemPageInfo,
  translated: { description?: string; input?: string; output?: string; hint?: string } | null,
  showingOriginal: boolean,
  status: string
) {
  const title = nonEmpty(info.title) ?? "Problem";
  const time = nonEmpty(info.time_limit) ?? "";
  const mem = nonEmpty(info.memory_limit) ?? "";

  const desc = nonEmpty(showingOriginal ? info.description : translated?.description ?? info.description) ?? "";
  const input = nonEmpty(showingOriginal ? info.input_spec : translated?.input ?? info.input_spec) ?? "";
  const output = nonEmpty(showingOriginal ? info.output_spec : translated?.output ?? info.output_spec) ?? "";
  const hint = nonEmpty(showingOriginal ? info.hint : translated?.hint ?? info.hint) ?? "";
  const sampleIn = nonEmpty(info.sample_input) ?? "";
  const sampleOut = nonEmpty(info.sample_output) ?? "";

  const rows: string[] = [];
  if (status.trim()) {
    rows.push(`<div class="muted">${escapeHtml(status)}</div>`);
  }
  rows.push(`<div><b>${escapeHtml(title)}</b></div>`);
  if (time || mem) {
    rows.push(`<div class="muted">${escapeHtml([time && `Time: ${time}`, mem && `Memory: ${mem}`].filter(Boolean).join(" | "))}</div>`);
  }

  const section = (label: string, body: string) => {
    if (!body.trim()) return;
    rows.push(`<h4>${escapeHtml(label)}</h4>`);
    rows.push(`<pre class="pre">${escapeHtml(body)}</pre>`);
  };

  section("Description", desc);
  section("Input", input);
  section("Output", output);
  section("Sample Input", sampleIn);
  section("Sample Output", sampleOut);
  section("Hint", hint);

  if (rows.length <= 2) {
    rows.push(`<div class="muted">No statement parsed.</div>`);
  }

  return rows.join("\\n");
}

function inferLangMode(label: string, value: string): CodeMode {
  const s = (label || value || "").trim().toLowerCase();
  return s.includes("python") || s === "py" ? "python" : "plain";
}

async function pickFavoriteFolder(services: AppServices): Promise<number | null> {
  const folders: FavoriteFolderInfo[] = await services.invoke("oj_favorite_list_folders");

  if (folders.length === 0) {
    const name = (window.prompt("Create a folder name:") ?? "").trim();
    if (!name) return null;
    const folderId: number = await services.invoke("oj_favorite_create_folder", { name });
    return folderId;
  }

  const names = folders.map((f) => f.name).join(", ");
  const typed = (window.prompt(`Save to folder (existing: ${names}). Enter folder name:`) ?? "").trim();
  if (!typed) return null;

  const existing = folders.find((f) => f.name === typed);
  if (existing) return existing.id;

  const folderId: number = await services.invoke("oj_favorite_create_folder", { name: typed });
  return folderId;
}

export function createProblemPage(services: AppServices, init: ProblemInit) {
  let toolsExpanded = true;
  let aiVisible = false;

  let info: ProblemPageInfo = init.info;
  let contestTitle = init.contestTitle;

  const titleLabel = el("div", { id: "problemTitleLabel" }, [nonEmpty(info.title) ?? "Problem"]);

  const homeBtn = iconButton(services, "problemTopHome", "Home", "homepage.svg");
  const themeBtn = iconButton(services, "problemTopTheme", "Dark Mode", "dark-mode.png");
  const refreshBtn = iconButton(services, "problemTopRefresh", "Refresh", "refresh.svg");

  const topFrame = el("div", { id: "problemTopFrame" }, [
    titleLabel,
    el("div", { id: "problemTopActions" }, [homeBtn, themeBtn, refreshBtn]),
  ]);

  // tools
  const toolsToggle = el("button", { id: "problemToolsToggleButton" }, ["Tools v"]);
  const backBtn = el("button", { id: "problemToolButton" }, ["Back"]);
  const favBtn = el("button", { id: "problemToolButton" }, ["Favorite Current Problem"]);
  const aiBtn = el("button", { id: "problemToolButton" }, ["AI"]);

  const toolsPanel = el("div", { id: "problemToolsPanel" }, [backBtn, favBtn, aiBtn]);

  const collapsedBack = iconButton(services, "problemCollapsedBack", "Back", "back.svg");
  const collapsedFav = iconButton(services, "problemCollapsedFavorite", "Favorite", "favorite.png");
  const collapsedAi = iconButton(services, "problemCollapsedAi", "AI", "ai-config.svg", "aiconfig.svg");
  const collapsedPanel = el("div", { id: "problemCollapsedToolsPanel" }, [collapsedBack, collapsedFav, collapsedAi]);

  const leftFrame = el("div", { id: "problemLeftFrame" }, [toolsToggle, toolsPanel, collapsedPanel]);

  // problem detail frame
  const showOriginalBtn = el("button", { id: "problemRefreshButton" }, ["Original"]);
  const translateBtn = el("button", { id: "problemRefreshButton" }, ["Translate"]);

  const problemHeader = el("div", { class: "rowline" }, [
    el("div", { id: "problemSectionLabel" }, ["Problem"]),
    el("div", { class: "tabs" }, [showOriginalBtn, translateBtn]),
  ]);

  const detail = el("div", { id: "problemDetailText" }, ["Loading problem detail..."]);
  const problemFrame = el("div", { id: "problemMiddleFrame" }, [problemHeader, detail]);

  // submit frame
  const submitLabel = el("div", { id: "problemSectionLabel" }, ["Submit"]);
  const languageSelect = document.createElement("select");
  languageSelect.id = "problemLanguageCombo";
  languageSelect.className = "select";

  const problemUrl = info.problem_url;

  const editor = makeEditor("");

  const testTabBtn = el("button", { class: "problemResultTabButton active" }, ["test"]);
  const submitTabBtn = el("button", { class: "problemResultTabButton" }, ["submit"]);

  const inputBtn = el("button", { id: "problemInputButton" }, ["input"]);
  const submitBtn = el("button", { id: "problemSubmitButton" }, ["Submit Code"]);

  const testInput = el("textarea", { id: "problemTestInput", class: "pre pre-textarea", placeholder: "Write test input here." }, []) as HTMLTextAreaElement;
  const testOutput = el("pre", { id: "problemTestOutput", class: "pre" }, [""]);

  const submitOutput = el("pre", { id: "problemSubmitOutput", class: "pre" }, ["Preparing submit options..."]);
  const testPaneHandle = el("div", { id: "problemTestPaneHandle", class: "splitter-handle v8" }, []);
  const testPane = el("div", { id: "problemTestPaneSplitter" }, [
    el("div", { class: "problemTestPane" }, [testInput]),
    testPaneHandle,
    el("div", { class: "problemTestPane" }, [testOutput]),
  ]);

  const resultStack = el("div", { id: "problemResultStack" }, [testPane, submitOutput]);

  const resultTabs = el("div", { class: "rowline" }, [
    el("div", { class: "tabs" }, [testTabBtn, submitTabBtn]),
    el("div", { class: "tabs" }, [inputBtn, submitBtn]),
  ]);
  const submitPaneHandle = el("div", { id: "problemSubmitPaneHandle", class: "splitter-handle h" }, []);
  const submitPaneSplitter = el("div", { id: "problemSubmitPaneSplitter" }, [
    editor.root,
    submitPaneHandle,
    el("div", { id: "problemResultPanel" }, [resultTabs, resultStack]),
  ]);

  const submitFrame = el("div", { id: "problemRightFrame" }, [submitLabel, languageSelect, submitPaneSplitter]);

  // ai frame
  const aiFrame = el("div", { id: "problemAiFrame" }, []);
  const aiLabel = el("div", { id: "problemSectionLabel" }, ["AI"]);
  const aiConfigLabel = el("div", { id: "problemAiMetaLabel" }, ["Config:"]);
  const aiResponseLabel = el("div", { id: "problemAiFieldLabel" }, ["Response"]);
  const aiResponse = el("div", { id: "problemResultText", class: "pre" }, [""]);
  const aiPromptLabel = el("div", { id: "problemAiFieldLabel" }, ["Prompt"]);
  const aiPrompt = el("textarea", { id: "problemAiPrompt", class: "pre pre-textarea", placeholder: "Ask AI about the current problem, code, or test result." }, []) as HTMLTextAreaElement;
  const aiAskBtn = el("button", { id: "problemSubmitButton" }, ["Ask"]);

  aiFrame.append(aiLabel, aiConfigLabel, aiResponseLabel, aiResponse, aiPromptLabel, aiPrompt, el("div", { class: "qtActionRow problemActionRow" }, [aiAskBtn]));
  (aiFrame as HTMLElement).style.display = "none";
  const workspaceHandle = el("div", { id: "problemWorkspaceHandle", class: "splitter-handle v4" }, []);
  const workspaceSplitter = el("div", { id: "problemWorkspaceSplitter" }, [submitFrame, workspaceHandle, aiFrame]);

  const contentHandle = el("div", { id: "problemContentHandle", class: "splitter-handle v4" }, []);
  const contentSplitter = el("div", { id: "problemContentSplitter" }, [problemFrame, contentHandle, workspaceSplitter]);

  const bottom = el("div", { id: "problemBottomRow" }, [leftFrame, contentSplitter]);
  const page = el("div", { id: "problemPage" }, [topFrame, bottom]);

  // state
  let showingOriginal = false;
  let translationLoading = false;
  let translation: { description?: string; input?: string; output?: string; hint?: string } | null = null;
  let translationStatus = "";

  let submitPage: SubmitPageInfo | null = null;
  let testing = false;
  let showTestTab = true;
  const setGridColsPx = (container: HTMLElement, leftPx: number, handlePx: number, rightPx: number) => {
    const l = Math.max(0, Math.round(leftPx));
    const r = Math.max(0, Math.round(rightPx));
    container.style.gridTemplateColumns = `${l}px ${handlePx}px ${r}px`;
  };

  const setGridRowsPx = (container: HTMLElement, topPx: number, handlePx: number, bottomPx: number) => {
    const t = Math.max(0, Math.round(topPx));
    const b = Math.max(0, Math.round(bottomPx));
    container.style.gridTemplateRows = `${t}px ${handlePx}px ${b}px`;
  };

  const attachColSplitter = (opts: {
    container: HTMLElement;
    handle: HTMLElement;
    left: HTMLElement;
    right: HTMLElement;
    handlePx: number;
    minLeftPx: number;
    minRightPx: number;
    enabled?: () => boolean;
  }) => {
    const { container, handle, left, right, handlePx, minLeftPx, minRightPx } = opts;
    const enabled = opts.enabled ?? (() => true);

    handle.addEventListener("pointerdown", (ev) => {
      if (!enabled()) return;
      if (ev.button !== 0) return;
      ev.preventDefault();

      const rect = container.getBoundingClientRect();
      const total = Math.max(0, rect.width - handlePx);

      let startLeft = left.getBoundingClientRect().width;
      if (!Number.isFinite(startLeft) || startLeft <= 0) startLeft = total * 0.5;

      const startX = ev.clientX;

      const move = (e: PointerEvent) => {
        const dx = e.clientX - startX;
        let nextLeft = startLeft + dx;
        nextLeft = Math.max(minLeftPx, Math.min(total - minRightPx, nextLeft));
        setGridColsPx(container, nextLeft, handlePx, total - nextLeft);
      };

      const up = () => {
        document.removeEventListener("pointermove", move);
        document.removeEventListener("pointerup", up);
        document.body.style.userSelect = "";
      };

      document.body.style.userSelect = "none";
      document.addEventListener("pointermove", move);
      document.addEventListener("pointerup", up);
    });
  };

  const attachRowSplitter = (opts: {
    container: HTMLElement;
    handle: HTMLElement;
    top: HTMLElement;
    bottom: HTMLElement;
    handlePx: number;
    minTopPx: number;
    minBottomPx: number;
  }) => {
    const { container, handle, top, bottom, handlePx, minTopPx, minBottomPx } = opts;

    handle.addEventListener("pointerdown", (ev) => {
      if (ev.button !== 0) return;
      ev.preventDefault();

      const rect = container.getBoundingClientRect();
      const total = Math.max(0, rect.height - handlePx);

      let startTop = top.getBoundingClientRect().height;
      if (!Number.isFinite(startTop) || startTop <= 0) startTop = total * 0.6;

      const startY = ev.clientY;

      const move = (e: PointerEvent) => {
        const dy = e.clientY - startY;
        let nextTop = startTop + dy;
        nextTop = Math.max(minTopPx, Math.min(total - minBottomPx, nextTop));
        setGridRowsPx(container, nextTop, handlePx, total - nextTop);
      };

      const up = () => {
        document.removeEventListener("pointermove", move);
        document.removeEventListener("pointerup", up);
        document.body.style.userSelect = "";
      };

      document.body.style.userSelect = "none";
      document.addEventListener("pointermove", move);
      document.addEventListener("pointerup", up);
    });
  };

  // Qt-like splitters (ProblemPage)
  attachColSplitter({
    container: contentSplitter as HTMLElement,
    handle: contentHandle as HTMLElement,
    left: problemFrame as HTMLElement,
    right: workspaceSplitter as HTMLElement,
    handlePx: 4,
    minLeftPx: 320,
    minRightPx: 420,
  });

  attachColSplitter({
    container: workspaceSplitter as HTMLElement,
    handle: workspaceHandle as HTMLElement,
    left: submitFrame as HTMLElement,
    right: aiFrame as HTMLElement,
    handlePx: 4,
    minLeftPx: 420,
    minRightPx: 320,
    enabled: () => aiVisible,
  });

  const submitResultPanel = submitPaneSplitter.querySelector("#problemResultPanel") as HTMLElement;
  attachRowSplitter({
    container: submitPaneSplitter as HTMLElement,
    handle: submitPaneHandle as HTMLElement,
    top: editor.root,
    bottom: submitResultPanel,
    handlePx: 8,
    minTopPx: 200,
    minBottomPx: 160,
  });

  const testLeftPane = testPane.children[0] as HTMLElement;
  const testRightPane = testPane.children[2] as HTMLElement;
  attachColSplitter({
    container: testPane as HTMLElement,
    handle: testPaneHandle as HTMLElement,
    left: testLeftPane,
    right: testRightPane,
    handlePx: 8,
    minLeftPx: 220,
    minRightPx: 220,
  });

  const setResultTab = (test: boolean) => {
    showTestTab = test;
    testTabBtn.classList.toggle("active", test);
    submitTabBtn.classList.toggle("active", !test);

    (testPane as HTMLElement).style.display = test ? "grid" : "none";
    (submitOutput as HTMLElement).style.display = test ? "none" : "block";

    (inputBtn as HTMLButtonElement).style.display = test ? "inline-flex" : "none";
    (submitBtn as HTMLButtonElement).style.display = test ? "none" : "inline-flex";

    const hasLang = (languageSelect as HTMLSelectElement).options.length > 0;
    (submitBtn as HTMLButtonElement).disabled = !(!test && hasLang);
    (inputBtn as HTMLButtonElement).disabled = !(test && hasLang && !testing);
  };

  const setSubmitEnabled = (enabled: boolean) => {
    const hasLang = languageSelect.options.length > 0;
    languageSelect.disabled = !(enabled && hasLang);
    editor.textarea.disabled = !(enabled && hasLang);
    setResultTab(showTestTab);
  };

  const setFavoriteEnabled = (enabled: boolean) => {
    (favBtn as HTMLButtonElement).disabled = !enabled;
    (collapsedFav as HTMLButtonElement).disabled = !enabled;
  };

  const setToolsExpanded = (expanded: boolean) => {
    toolsExpanded = expanded;
    if (toolsExpanded) {
      leftFrame.classList.remove("collapsed");
      (toolsPanel as HTMLElement).style.display = "grid";
      (collapsedPanel as HTMLElement).style.display = "none";
      toolsToggle.textContent = "Tools v";
      toolsToggle.title = "Collapse Tools";
    } else {
      leftFrame.classList.add("collapsed");
      (toolsPanel as HTMLElement).style.display = "none";
      (collapsedPanel as HTMLElement).style.display = "grid";
      toolsToggle.textContent = ">";
      toolsToggle.title = "Expand Tools";
    }
  };
  const setAiPanelVisible = (visible: boolean) => {
    aiVisible = visible;

    const handlePx = 4;

    if (!visible) {
      (aiFrame as HTMLElement).style.display = "none";
      (workspaceHandle as HTMLElement).style.pointerEvents = "none";
      workspaceSplitter.classList.remove("ai-visible");
      (workspaceSplitter as HTMLElement).style.gridTemplateColumns = "1fr 0px 0px";
      return;
    }

    (aiFrame as HTMLElement).style.display = "grid";
    (workspaceHandle as HTMLElement).style.pointerEvents = "auto";
    workspaceSplitter.classList.add("ai-visible");
    const rect = (workspaceSplitter as HTMLElement).getBoundingClientRect();
    const total = Math.max(0, rect.width - handlePx);

    let leftW = submitFrame.getBoundingClientRect().width;
    let rightW = aiFrame.getBoundingClientRect().width;

    if (!Number.isFinite(leftW) || leftW <= 0) leftW = Math.max(420, total - 320);
    if (!Number.isFinite(rightW) || rightW <= 0) {
      leftW = Math.max(420, leftW);
      rightW = 320;
    }

    if (leftW + rightW > total) {
      leftW = Math.max(420, total - rightW);
      rightW = Math.max(0, total - leftW);
    }

    setGridColsPx(workspaceSplitter as HTMLElement, leftW, handlePx, Math.max(0, total - leftW));
  };

  const refreshDetail = () => {
    detail.innerHTML = renderProblemDetailHtml(info, translation, showingOriginal, translationStatus);
  };

  const applyCachedTranslation = () => {
    const d = localStorage.getItem(translationKey(problemUrl, "description")) ?? "";
    const i = localStorage.getItem(translationKey(problemUrl, "input")) ?? "";
    const o = localStorage.getItem(translationKey(problemUrl, "output")) ?? "";
    const h = localStorage.getItem(translationKey(problemUrl, "hint")) ?? "";
    const has = !!(d.trim() || i.trim() || o.trim() || h.trim());
    translation = has ? { description: d, input: i, output: o, hint: h } : null;
    translationStatus = has ? "Showing cached translation." : "";
    showingOriginal = false;
    refreshDetail();
  };

  const setProblemTranslating = (loading: boolean) => {
    translationLoading = loading;
    const canTranslate = !!(
      nonEmpty(info.description) ||
      nonEmpty(info.input_spec) ||
      nonEmpty(info.output_spec) ||
      nonEmpty(info.hint)
    );

    (showOriginalBtn as HTMLButtonElement).disabled = !(translation && !loading);
    (translateBtn as HTMLButtonElement).disabled = !(canTranslate && !loading);

    if (loading) {
      translateBtn.textContent = "Translating...";
      translationStatus = "Translating description, input, output, and hint...";
    } else if (translation) {
      translateBtn.textContent = "Translated";
    } else {
      translateBtn.textContent = "Translate";
    }

    refreshDetail();
  };

  const translateProblem = async () => {
    if (translation && !showingOriginal) {
      translationStatus = "Showing cached translation.";
      refreshDetail();
      return;
    }

    setProblemTranslating(true);

    try {
      const prompt =
        [
          "You are a translation engine.",
          "Translate the provided fields into Simplified Chinese.",
          "Only translate natural-language description; do NOT translate code, variable names, URLs, or sample IO.",
          "Keep line breaks.",
          "Return ONLY a JSON object with keys: description, input, output, hint.",
          "---",
          `description:
${info.description ?? ""}`,
          `input:
${info.input_spec ?? ""}`,
          `output:
${info.output_spec ?? ""}`,
          `hint:
${info.hint ?? ""}`,
        ].join("\\n");

      const messages: AiChatMessage[] = [{ role: "user", content: prompt }];
      const resp: string = await services.invoke("oj_ai_chat", { messages });
      const obj = parseJsonObject(resp);

      const d = String(obj?.description ?? "");
      const i = String(obj?.input ?? "");
      const o = String(obj?.output ?? "");
      const h = String(obj?.hint ?? "");

      localStorage.setItem(translationKey(problemUrl, "description"), d);
      localStorage.setItem(translationKey(problemUrl, "input"), i);
      localStorage.setItem(translationKey(problemUrl, "output"), o);
      localStorage.setItem(translationKey(problemUrl, "hint"), h);

      applyCachedTranslation();
      translationStatus = "Translation applied.";
    } catch (e) {
      translationStatus = "Translation failed: " + String(e);
    } finally {
      setProblemTranslating(false);
    }
  };

  const resetSubmitPanel = () => {
    testing = false;
    submitPage = null;

    languageSelect.replaceChildren();
    languageSelect.disabled = true;

    editor.textarea.value = "";
    editor.sync();
    editor.textarea.disabled = true;

    testOutput.textContent = "";
    submitOutput.textContent = "Preparing submit options...";

    setResultTab(true);
  };

  const restoreDraftOrStarterCode = () => {
    const draft = localStorage.getItem(draftKey(problemUrl)) ?? "";
    if (draft.trim()) {
      editor.textarea.value = draft;
      editor.sync();
      return;
    }

    const starter = nonEmpty(info.starter_code) ?? "";
    editor.textarea.value = starter;
    editor.sync();
  };

  const refreshEditorLanguageMode = () => {
    const opt = languageSelect.selectedOptions[0];
    const label = opt?.textContent ?? "";
    const value = String(languageSelect.value ?? "");
    editor.setMode(inferLangMode(label, value));
  };

  const showSubmitPageLoaded = (submit: SubmitPageInfo) => {
    const opts = submit.languages ?? [];
    languageSelect.replaceChildren();

    for (const opt of opts as SubmitLanguageOption[]) {
      const o = document.createElement("option");
      o.value = opt.value;
      o.textContent = opt.label || opt.value;
      if (opt.checked) o.selected = true;
      languageSelect.append(o);
    }

    // preferred language
    const cachedLang = localStorage.getItem(langKey(problemUrl)) ?? "";
    if (cachedLang) {
      const exists = Array.from(languageSelect.options).some((o) => o.value === cachedLang);
      if (exists) languageSelect.value = cachedLang;
    }

    const hasLang = languageSelect.options.length > 0;
    languageSelect.disabled = !hasLang;
    editor.textarea.disabled = !hasLang;

    if (hasLang) {
      localStorage.setItem(langKey(problemUrl), languageSelect.value);
    }

    refreshEditorLanguageMode();

    submitOutput.textContent = `Submit page loaded.
Action: ${submit.submit_action_url ?? ""}`.trim();

    setSubmitEnabled(true);
  };

  const loadSubmit = async () => {
    resetSubmitPanel();
    restoreDraftOrStarterCode();

    const submitUrl = nonEmpty(info.submit_url);
    if (!submitUrl) {
      submitOutput.textContent = "Submit Status: submit url not found.";
      return;
    }

    submitOutput.textContent = "Preparing submit options...";
    try {
      submitPage = await services.invoke("oj_open_submit", { submitPageUrl: submitUrl });
      showSubmitPageLoaded(submitPage);
    } catch (e) {
      submitOutput.textContent = "Submit Status: " + String(e);
    }
  };

  const submitSolution = async () => {
    setResultTab(false);

    if (!submitPage) {
      await loadSubmit();
      if (!submitPage) return;
    }

    const hasLang = languageSelect.options.length > 0;
    if (!hasLang) {
      submitOutput.textContent = "Submit Status: Missing language.";
      return;
    }

    submitOutput.textContent = "Submitting...";

    try {
      const resp: SubmitResponse = await services.invoke("oj_submit_solution", {
        submitPage: submitPage,
        language: languageSelect.value,
        sourceText: editor.textarea.value,
      });

      submitOutput.textContent = `Submit Status: ${resp.ok ? "OK" : "Failed"} (${resp.status_code})
Final: ${resp.final_url}`;

      if (resp.inferred_result_url) {
        const result: ResultPageInfo = await services.invoke("oj_open_result", {
          resultPageUrl: resp.inferred_result_url,
        });

        const lines = [
          `Judge Status: ${result.status_text ?? "<none>"}`,
          result.detail_title ? `
${result.detail_title}` : "",
          result.detail_text ? `
${result.detail_text}` : "",
        ].join("");

        submitOutput.textContent = lines.trim();

        // poll when waiting (Qt behavior)
        try {
          const waiting: boolean = await services.invoke("oj_result_is_waiting", { result });
          if (waiting) {
            window.setTimeout(async () => {
              try {
                const r2: ResultPageInfo = await services.invoke("oj_open_result", { resultPageUrl: result.page_url });
                submitOutput.textContent =
                  `Judge Status: ${r2.status_text ?? "<none>"}` +
                  (r2.detail_title ? `

${r2.detail_title}` : "") +
                  (r2.detail_text ? `
${r2.detail_text}` : "");
              } catch {
                // ignore
              }
            }, 2000);
          }
        } catch {
          // ignore
        }
      }
    } catch (e) {
      submitOutput.textContent = "Submit Status: " + String(e);
    }
  };

  const runTest = async () => {
    setResultTab(true);
    testing = true;
    setResultTab(true);

    testOutput.textContent = "Running test...";

    try {
      const opt = languageSelect.selectedOptions[0];
      const label = opt?.textContent ?? languageSelect.value;
      const mode = inferLangMode(String(label ?? ""), String(languageSelect.value ?? ""));

      if (languageSelect.options.length === 0) {
        testOutput.textContent = "No language selected.";
        return;
      }

      const lang = mode === "python" ? "python" : "cpp";
      const fileName = lang === "python" ? "main.py" : "main.cpp";

      const resp: JudgeResponse = await services.invoke("oj_judge_source", {
        language: lang,
        fileName,
        sourceCode: editor.textarea.value,
        stdinText: testInput.value,
      });

      testOutput.textContent = resp.body;
    } catch (e) {
      testOutput.textContent = "Test failed: " + String(e);
    } finally {
      testing = false;
      setResultTab(true);
    }
  };

  // events
  toolsToggle.addEventListener("click", () => setToolsExpanded(!toolsExpanded));
  backBtn.addEventListener("click", () => services.router.pop());
  collapsedBack.addEventListener("click", () => services.router.pop());

  favBtn.addEventListener("click", async () => {
    try {
      const folderId = await pickFavoriteFolder(services);
      if (!folderId) return;
      await services.invoke("oj_favorite_save_to_folder", { folder_id: folderId, problem: info });
      services.setGlobalStatus(true, "Saved to favorites.");
    } catch (e) {
      services.setGlobalStatus(false, String(e));
    }
  });
  collapsedFav.addEventListener("click", async () => {
    (favBtn as HTMLButtonElement).click();
  });

  aiBtn.addEventListener("click", () => setAiPanelVisible(!aiVisible));
  collapsedAi.addEventListener("click", () => setAiPanelVisible(!aiVisible));

  testTabBtn.addEventListener("click", () => setResultTab(true));
  submitTabBtn.addEventListener("click", () => setResultTab(false));

  inputBtn.addEventListener("click", () => void runTest());
  submitBtn.addEventListener("click", () => void submitSolution());

  showOriginalBtn.addEventListener("click", () => {
    if (!translation) return;
    showingOriginal = !showingOriginal;
    translationStatus = showingOriginal ? "Showing original text." : "Showing cached translation.";
    refreshDetail();
  });

  translateBtn.addEventListener("click", () => void translateProblem());

  languageSelect.addEventListener("change", () => {
    localStorage.setItem(langKey(problemUrl), languageSelect.value);
    refreshEditorLanguageMode();
    setResultTab(showTestTab);
  });

  editor.textarea.addEventListener("input", () => {
    localStorage.setItem(draftKey(problemUrl), editor.textarea.value);
  });

  aiAskBtn.addEventListener("click", async () => {
    const q = aiPrompt.value.trim();
    if (!q) return;

    aiAskBtn.setAttribute("disabled", "true");
    try {
      const messages: AiChatMessage[] = [{ role: "user", content: q }];
      const resp: string = await services.invoke("oj_ai_chat", { messages });
      aiResponse.textContent = `User
${q}

Assistant
${resp}`;
    } catch (e) {
      aiResponse.textContent = "AI failed: " + String(e);
    } finally {
      aiAskBtn.removeAttribute("disabled");
    }
  });

  refreshBtn.addEventListener("click", async () => {
    try {
      const p: ProblemPageInfo = await services.invoke("oj_open_problem", { problemUrl: info.problem_url });
      info = p;
      titleLabel.textContent = nonEmpty(info.title) ?? "Problem";
      translation = null;
      translationStatus = "";
      showingOriginal = false;
      applyCachedTranslation();
      resetSubmitPanel();
      await loadSubmit();
      services.setGlobalStatus(true, "");
    } catch (e) {
      services.setGlobalStatus(false, String(e));
    }
  });

  themeBtn.addEventListener("click", () => {
    services.setTheme(services.getTheme() === "dark" ? "light" : "dark");
    themeBtn.title = services.getTheme() === "dark" ? "Light Mode" : "Dark Mode";
  });

  homeBtn.addEventListener("click", () => services.goHome());

  window.addEventListener("keydown", (ev) => {
    const e = ev as KeyboardEvent;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "r") {
      e.preventDefault();
      void refreshBtn.click();
    }
  });

  // init
  setToolsExpanded(true);
  setAiPanelVisible(false);

  // translation cache
  applyCachedTranslation();
  setProblemTranslating(false);

  // submit/init
  resetSubmitPanel();
  restoreDraftOrStarterCode();
  setFavoriteEnabled(true);
  setSubmitEnabled(false);
  setResultTab(true);

  // load AI config summary (Qt shows Config: ...)
  void (async () => {
    try {
      const cfg: any = await services.invoke("get_openai_config");
      const model = String(cfg?.model ?? "");
      const base = String(cfg?.base_url ?? "");
      aiConfigLabel.textContent = model || base ? `Config: ${model}${model && base ? " | " : ""}${base}` : "Config:";
    } catch {
      // ignore
    }
  })();

  return {
    id: "problem" as const,
    el: page,
    onShow: () => loadSubmit(),
    onHide: () => {
      // save draft explicitly (Qt saves when switching problems)
      localStorage.setItem(draftKey(problemUrl), editor.textarea.value);
      localStorage.setItem(langKey(problemUrl), languageSelect.value);
    },
  };
}
