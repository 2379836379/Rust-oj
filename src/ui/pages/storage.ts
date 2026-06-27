import type { AppServices, IconName } from "../app";
import { el } from "../dom";
import type { AppState, StorageSizes } from "./types";

type StorageInit = {
  // reserved
};

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

function formatBytes(bytes: number): string {
  const b = Number(bytes || 0);
  if (!Number.isFinite(b) || b <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = b;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  const fixed = i === 0 ? String(Math.round(v)) : v.toFixed(v >= 10 ? 1 : 2);
  return `${fixed} ${units[i]}`;
}

function playRingPath(ringPath: string) {
  const ring = String(ringPath ?? "").trim();
  if (!ring) throw new Error("No file selected");
  const normalized = ring.replace(/\\/g, "/");
  const url = normalized.startsWith("/") ? "file://" + normalized : "file:///" + normalized;
  return new Audio(url).play();
}

export function createStoragePage(services: AppServices, _init?: StorageInit) {
  let toolsExpanded = true;
  let alarmEnabled = false;
  let ringPath = "";

  const titleLabel = el("div", { id: "storageTitleLabel" }, ["Set"]);

  const homeBtn = iconButton(services, "storageTopHome", "Home", "homepage.svg");
  const themeBtn = iconButton(services, "storageTopTheme", "Dark Mode", "dark-mode.png");

  const topFrame = el("div", { id: "storageTopFrame" }, [
    titleLabel,
    el("div", { id: "storageTopActions" }, [homeBtn, themeBtn]),
  ]);

  const toolsToggle = el("button", { id: "storageToolsToggleButton" }, ["Tools v"]);
  const backBtn = el("button", { id: "storageToolButton" }, ["Back"]);
  const toolsPanel = el("div", { id: "storageToolsPanel" }, [backBtn]);

  const collapsedBack = iconButton(services, "storageCollapsedBack", "Back", "back.svg");
  const collapsedPanel = el("div", { id: "storageCollapsedToolsPanel" }, [collapsedBack]);

  const leftFrame = el("div", { id: "storageLeftFrame" }, [toolsToggle, toolsPanel, collapsedPanel]);

  const cacheSizeLabel = el("div", { class: "storageLabel", "data-obj": "storageLabel" }, ["Cache Size"]);
  const cacheSizeValue = el("div", { class: "storageValueLabel", "data-obj": "storageValueLabel" }, ["--"]);
  const appSizeLabel = el("div", { class: "storageLabel", "data-obj": "storageLabel" }, ["App Data Size"]);
  const appSizeValue = el("div", { class: "storageValueLabel", "data-obj": "storageValueLabel" }, ["--"]);

  const alarmLabel = el("div", { class: "storageLabel", "data-obj": "storageLabel" }, ["Alarm"]);
  const alarmToggle = el("input", { id: "storageAlarmToggle", type: "checkbox" }, []) as HTMLInputElement;
  const alarmToggleWrap = el("label", { class: "switch" }, [alarmToggle, el("span", { class: "slider" }, [])]);

  const ringLabel = el("div", { class: "storageLabel", "data-obj": "storageLabel" }, ["Ring"]);
  const ringPathValue = el("div", { id: "storagePathLabel" }, ["No file selected"]);

  const alarmTestBtn = el("button", { class: "storageClearButton", "data-obj": "storageClearButton" }, ["Alarm Test"]);
  const ringPickBtn = el("button", { class: "storageClearButton", "data-obj": "storageClearButton" }, ["Pick Ring Path"]);
  const clearCacheBtn = el("button", { class: "storageClearButton", "data-obj": "storageClearButton" }, ["Clear Cache"]);

  const statusLabel = el("div", { id: "storageStatusLabel" }, [""]);

  const row = (left: HTMLElement, right: HTMLElement) =>
    el("div", { class: "storageRow" }, [left, right]);

  const contentFrame = el("div", { id: "storageContentFrame" }, [
    row(cacheSizeLabel, cacheSizeValue),
    row(appSizeLabel, appSizeValue),
    el("hr", {}, []),
    row(alarmLabel, alarmToggleWrap),
    row(ringLabel, ringPathValue),
    el("div", { class: "qtActionRow" }, [alarmTestBtn, ringPickBtn, clearCacheBtn]),
    statusLabel,
  ]);

  const bottom = el("div", { id: "storageBottomRow" }, [leftFrame, contentFrame]);
  const page = el("div", { id: "storagePage" }, [topFrame, bottom]);

  const applyToolsExpanded = () => {
    if (toolsExpanded) {
      leftFrame.classList.remove("collapsed");
      toolsPanel.style.display = "grid";
      collapsedPanel.style.display = "none";
      toolsToggle.textContent = "Tools v";
      toolsToggle.title = "Collapse Tools";
    } else {
      leftFrame.classList.add("collapsed");
      toolsPanel.style.display = "none";
      collapsedPanel.style.display = "grid";
      toolsToggle.textContent = ">";
      toolsToggle.title = "Expand Tools";
    }
  };

  const setAlarmEnabled = (enabled: boolean) => {
    alarmEnabled = enabled;
    alarmToggle.checked = enabled;
  };

  const setRingPath = (path: string) => {
    ringPath = path;
    ringPathValue.textContent = path.trim() ? path : "No file selected";
  };

  const load = async () => {
    try {
      const st: AppState = await services.invoke("get_app_state");
      setAlarmEnabled(!!st.alarm_enabled);
      setRingPath(String(st.ring_path ?? ""));

      const sizes: StorageSizes = await services.invoke("oj_storage_get_sizes");
      cacheSizeValue.textContent = formatBytes(sizes.cache_bytes);
      appSizeValue.textContent = formatBytes(sizes.app_bytes);
      statusLabel.textContent = "";
      services.setGlobalStatus(true, "");
    } catch (e) {
      statusLabel.textContent = String(e);
      services.setGlobalStatus(false, String(e));
    }
  };

  const saveAlarmState = async () => {
    try {
      const input = { ring_path: ringPath || null, alarm_enabled: alarmEnabled };
      await services.invoke("set_app_state", { state: input });
      services.setGlobalStatus(true, "");
    } catch (e) {
      services.setGlobalStatus(false, String(e));
    }
  };

  const clearCache = async () => {
    try {
      clearCacheBtn.setAttribute("disabled", "true");
      statusLabel.textContent = "Clearing cache...";
      const sizes: StorageSizes = await services.invoke("oj_storage_clear_cache");
      cacheSizeValue.textContent = formatBytes(sizes.cache_bytes);
      appSizeValue.textContent = formatBytes(sizes.app_bytes);
      statusLabel.textContent = "Cache cleared.";
      services.setGlobalStatus(true, "Cache cleared.");
    } catch (e) {
      statusLabel.textContent = String(e);
      services.setGlobalStatus(false, String(e));
    } finally {
      clearCacheBtn.removeAttribute("disabled");
    }
  };

  const alarmTest = async () => {
    try {
      statusLabel.textContent = "Playing ring...";
      await playRingPath(ringPath);
      statusLabel.textContent = "";
    } catch (e) {
      statusLabel.textContent = String(e);
    }
  };

  const pickRing = async () => {
    const v = (window.prompt("Enter ring file path:", ringPath) ?? "").trim();
    if (!v) return;
    setRingPath(v);
    await saveAlarmState();
  };

  toolsToggle.addEventListener("click", () => {
    toolsExpanded = !toolsExpanded;
    applyToolsExpanded();
  });

  backBtn.addEventListener("click", () => services.router.pop());
  collapsedBack.addEventListener("click", () => services.router.pop());

  alarmToggle.addEventListener("change", async () => {
    setAlarmEnabled(alarmToggle.checked);
    await saveAlarmState();
  });

  alarmTestBtn.addEventListener("click", () => void alarmTest());
  ringPickBtn.addEventListener("click", () => void pickRing());
  clearCacheBtn.addEventListener("click", () => void clearCache());

  themeBtn.addEventListener("click", () => {
    services.setTheme(services.getTheme() === "dark" ? "light" : "dark");
    themeBtn.title = services.getTheme() === "dark" ? "Light Mode" : "Dark Mode";
  });

  homeBtn.addEventListener("click", () => services.goHome());

  applyToolsExpanded();

  return {
    id: "storage" as const,
    el: page,
    onShow: () => load(),
  };
}