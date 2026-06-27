import type { AppServices, IconName } from "../app";
import { el } from "../dom";
import type { ContestPageInfo } from "./types";

import { createProblemPage } from "./problem";

type ContestInit = {
  title: string;
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

export function createContestPage(services: AppServices, init: ContestInit) {
  let toolsExpanded = true;
  let contestUrl = init.url;
  let contestTitle = init.title;

  const titleLabel = el("div", { id: "contestTitleLabel" }, [contestTitle || "Contest Set"]);

  const homeBtn = iconButton(services, "contestTopHome", "Home", "homepage.svg");
  const themeBtn = iconButton(services, "contestTopTheme", "Dark Mode", "dark-mode.png");
  const refreshBtn = iconButton(services, "contestTopRefresh", "Refresh", "refresh.svg");

  const topFrame = el("div", { id: "contestTopFrame" }, [
    titleLabel,
    el("div", { id: "contestTopActions" }, [homeBtn, themeBtn, refreshBtn]),
  ]);

  const toolsToggle = el("button", { id: "contestToolsToggleButton" }, ["Tools v"]);
  const backBtn = el("button", { id: "contestToolButton" }, ["Back"]);

  const toolsPanel = el("div", { id: "contestToolsPanel" }, [backBtn]);

  const collapsedBack = iconButton(services, "contestCollapsedBack", "Back", "back.svg");
  const collapsedPanel = el("div", { id: "contestCollapsedToolsPanel" }, [collapsedBack]);

  const leftFrame = el("div", { id: "contestLeftFrame" }, [toolsToggle, toolsPanel, collapsedPanel]);

  const infoLine = el("div", { id: "contestInfoLabel" }, ["Problems"]);
  const problemList = el("ul", { id: "contestProblemList", class: "list" }, []);
  const contentFrame = el("div", { id: "contestContentFrame" }, [
    el("div", { id: "contestSectionLabel" }, ["Problem List"]),
    infoLine,
    problemList,
  ]);

  const bottom = el("div", { id: "contestBottomRow" }, [leftFrame, contentFrame]);

  const page = el("div", { id: "contestPage" }, [topFrame, bottom]);

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
    problemList.replaceChildren(el("li", { class: "muted" }, ["Loading problems..."]));
  };

  const showError = (msg: string) => {
    problemList.replaceChildren(el("li", { class: "muted err" }, [msg]));
  };

  const render = (contest: ContestPageInfo) => {
    problemList.replaceChildren();
    infoLine.textContent = `Problems: ${contestTitle} (${contest.solved_problems}/${contest.total_problems})`;

    if (!contest.problems || contest.problems.length === 0) {
      problemList.append(el("li", { class: "muted" }, ["No problems."]));
      return;
    }

    for (const p of contest.problems) {
      const titleText = `${p.problem_id} - ${p.title}`;
      const row = el("div", { class: "rowline" }, [
        el("span", {}, [titleText]),
        p.solved ? el("span", { class: "tag finished" }, ["finished"]) : el("span", {}, []),
      ]);
      const li = el("li", { class: "list-item" }, [row]);
      li.addEventListener("click", async () => {
        try {
          const info: any = await services.invoke("oj_open_problem", { problemUrl: p.problem_url });
          services.router.push(createProblemPage(services, { info, contestTitle }));
        } catch (e) {
          services.setGlobalStatus(false, String(e));
        }
      });
      problemList.append(li);
    }
  };

  const load = async () => {
    showLoading();
    try {
      const contest: ContestPageInfo = await services.invoke("oj_open_contest", {
        contestPageUrl: contestUrl,
      });
      render(contest);
      services.setGlobalStatus(true, "");
    } catch (e) {
      showError(String(e));
      services.setGlobalStatus(false, String(e));
    }
  };

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
    id: "contest" as const,
    el: page,
    onShow: () => load(),
  };
}
