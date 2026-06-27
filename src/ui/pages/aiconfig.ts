import type { AppServices, IconName } from "../app";
import { el } from "../dom";
import type { OpenAiConfigText } from "./types";

type AiConfigInit = {
  // reserved for future
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

export function createAiConfigPage(services: AppServices, _init?: AiConfigInit) {
  let toolsExpanded = true;

  const titleLabel = el("div", { id: "aiConfigTitleLabel" }, ["AI Config"]);

  const homeBtn = iconButton(services, "aiConfigTopHome", "Home", "homepage.svg");
  const themeBtn = iconButton(services, "aiConfigTopTheme", "Dark Mode", "dark-mode.png");

  const topFrame = el("div", { id: "aiConfigTopFrame" }, [
    titleLabel,
    el("div", { id: "aiConfigTopActions" }, [homeBtn, themeBtn]),
  ]);

  const toolsToggle = el("button", { id: "aiConfigToolsToggleButton" }, ["Tools v"]);
  const backBtn = el("button", { id: "aiConfigToolButton" }, ["Back"]);
  const toolsPanel = el("div", { id: "aiConfigToolsPanel" }, [backBtn]);

  const collapsedBack = iconButton(services, "aiConfigCollapsedBack", "Back", "back.svg");
  const collapsedPanel = el("div", { id: "aiConfigCollapsedToolsPanel" }, [collapsedBack]);

  const leftFrame = el("div", { id: "aiConfigLeftFrame" }, [toolsToggle, toolsPanel, collapsedPanel]);

  const pathTitle = el("div", { class: "aiConfigFieldLabel", "data-obj": "aiConfigFieldLabel" }, ["Config File"]);
  const pathLabel = el("div", { id: "aiConfigPathLabel" }, ["--"]);

  const editorLabel = el("div", { class: "aiConfigFieldLabel", "data-obj": "aiConfigFieldLabel" }, ["config.toml"]);
  const edit = el(
    "textarea",
    { id: "aiConfigPromptEdit", class: "pre pre-textarea", placeholder: "[openai]\n..." },
    []
  ) as HTMLTextAreaElement;

  const statusLabel = el("div", { id: "aiConfigStatusLabel", class: "muted" }, [""]);
  const saveBtn = el("button", { id: "aiConfigSaveButton" }, ["Save"]);

  const contentFrame = el("div", { id: "aiConfigContentFrame" }, [
    pathTitle,
    pathLabel,
    editorLabel,
    edit,
    statusLabel,
    saveBtn,
  ]);

  const bottom = el("div", { id: "aiConfigBottomRow" }, [leftFrame, contentFrame]);
  const page = el("div", { id: "aiConfigPage" }, [topFrame, bottom]);

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

  const load = async () => {
    try {
      const cfg: OpenAiConfigText = await services.invoke("oj_aiconfig_get_text");
      pathLabel.textContent = cfg.path || "config.toml";
      edit.value = cfg.content ?? "";
      statusLabel.textContent = "";
      services.setGlobalStatus(true, "");
    } catch (e) {
      statusLabel.textContent = String(e);
      services.setGlobalStatus(false, String(e));
    }
  };

  const save = async () => {
    try {
      saveBtn.setAttribute("disabled", "true");
      statusLabel.textContent = "Saving...";
      const path: string = await services.invoke("oj_aiconfig_save_text", { content: edit.value });
      pathLabel.textContent = path || pathLabel.textContent;
      statusLabel.textContent = "Saved.";
      services.setGlobalStatus(true, "Saved.");
    } catch (e) {
      statusLabel.textContent = String(e);
      services.setGlobalStatus(false, String(e));
    } finally {
      saveBtn.removeAttribute("disabled");
    }
  };

  toolsToggle.addEventListener("click", () => {
    toolsExpanded = !toolsExpanded;
    applyToolsExpanded();
  });

  backBtn.addEventListener("click", () => services.router.pop());
  collapsedBack.addEventListener("click", () => services.router.pop());

  saveBtn.addEventListener("click", () => void save());

  themeBtn.addEventListener("click", () => {
    services.setTheme(services.getTheme() === "dark" ? "light" : "dark");
    themeBtn.title = services.getTheme() === "dark" ? "Light Mode" : "Dark Mode";
  });

  homeBtn.addEventListener("click", () => services.goHome());

  applyToolsExpanded();

  return {
    id: "aiconfig" as const,
    el: page,
    onShow: () => load(),
  };
}