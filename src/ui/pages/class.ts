import type { AppServices, IconName } from "../app";
import { el } from "../dom";
import type { ContestSetInfo, OpenClassResult } from "./types";

import { createContestPage } from "./contest";

type ClassInit = {
  name: string;
  url: string;
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

export function createClassPage(services: AppServices, init: ClassInit) {
  let toolsExpanded = true;
  let classUrl = init.url;
  let className = init.name;

  const titleLabel = el("div", { id: "classTitleLabel" }, [className || "Course"]);

  const homeBtn = iconButton(services, "classTopHome", "Home", "homepage.svg");
  const themeBtn = iconButton(services, "classTopTheme", "Dark Mode", "dark-mode.png");
  const refreshBtn = iconButton(services, "classTopRefresh", "Refresh", "refresh.svg");

  const topFrame = el("div", { id: "classTopFrame" }, [
    titleLabel,
    el("div", { id: "classTopActions" }, [homeBtn, themeBtn, refreshBtn]),
  ]);

  const toolsToggle = el("button", { id: "classToolsToggleButton" }, ["Tools v"]);
  const backBtn = el("button", { id: "classToolButton" }, ["Back"]);

  const toolsPanel = el("div", { id: "classToolsPanel" }, [backBtn]);

  const collapsedBack = iconButton(services, "classCollapsedBack", "Back", "back.svg");
  const collapsedPanel = el("div", { id: "classCollapsedToolsPanel" }, [collapsedBack]);

  const leftFrame = el("div", { id: "classLeftFrame" }, [toolsToggle, toolsPanel, collapsedPanel]);

  const contestList = el("ul", { id: "classContestList", class: "list" }, []);
  const contentFrame = el("div", { id: "classContentFrame" }, [
    el("div", { id: "classSectionLabel" }, ["Contest List"]),
    contestList,
  ]);

  const bottom = el("div", { id: "classBottomRow" }, [leftFrame, contentFrame]);

  const page = el("div", { id: "classPage" }, [topFrame, bottom]);

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

  const showLoading = () => {
    contestList.replaceChildren(el("li", { class: "muted" }, ["Loading contests..."]));
  };

  const showError = (msg: string) => {
    contestList.replaceChildren(el("li", { class: "muted err" }, [msg]));
  };

  const renderContestSets = (sets: ContestSetInfo[]) => {
    contestList.replaceChildren();
    if (!sets || sets.length === 0) {
      contestList.append(el("li", { class: "muted" }, ["No contests."]));
      return;
    }

    for (const cs of sets) {
      let text = cs.title ?? "";
      if (cs.extra_text) text += " | " + cs.extra_text;
      const li = el("li", { class: "list-item" }, [el("div", {}, [text])]);
      li.addEventListener("click", () => {
        const title = cs.title ?? "Contest Set";
        const url = cs.url;
        services.router.push(createContestPage(services, { title, url }));
      });
      contestList.append(li);
    }
  };

  const load = async () => {
    showLoading();
    try {
      const opened: OpenClassResult = await services.invoke("oj_open_class", {
        classPageUrl: classUrl,
      });

      const name = opened.class_info.course_name ?? className;
      className = name;
      titleLabel.textContent = className || "Course";

      renderContestSets(opened.group_info.contest_sets ?? []);
      services.setGlobalStatus(true, "");
    } catch (e) {
      showError(String(e));
      services.setGlobalStatus(false, String(e));
    }
  };

  // events
  toolsToggle.addEventListener("click", () => {
    toolsExpanded = !toolsExpanded;
    applyToolsExpanded();
  });

  backBtn.addEventListener("click", () => services.router.pop());
  collapsedBack.addEventListener("click", () => services.router.pop());

  refreshBtn.addEventListener("click", () => void load());
  window.addEventListener("keydown", (ev) => {
    const e = ev as KeyboardEvent;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "r") {
      e.preventDefault();
      void load();
    }
  });

  themeBtn.addEventListener("click", () => {
    services.setTheme(services.getTheme() === "dark" ? "light" : "dark");
    themeBtn.title = services.getTheme() === "dark" ? "Light Mode" : "Dark Mode";
  });

  homeBtn.addEventListener("click", () => services.goHome());

  applyToolsExpanded();

  return {
    id: "class" as const,
    el: page,
    onShow: () => load(),
  };
}
