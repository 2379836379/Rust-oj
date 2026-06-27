import type { AppServices, IconName } from "../app";
import { el } from "../dom";
import type { DeadlineReminder, JoinedClassInfo } from "./types";

import { createClassPage } from "./class";
import { createFavoritePage } from "./favorite";
import { createStoragePage } from "./storage";
import { createAiConfigPage } from "./aiconfig";
import { createContestPage } from "./contest";

type HomeInitial = {
  classes: JoinedClassInfo[];
  reminders: DeadlineReminder[];
  onUpdateReminders: (classes: JoinedClassInfo[], reminders: DeadlineReminder[]) => void;
  onLogout: () => void;
  onOpenClass: (name: string, url: string) => void;
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

function toolButton(text: string) {
  // Qt 里多个按钮同 objectName=homeToolButton；Web 里用 data-obj 模拟，避免重复 id。
  return el("button", { class: "homeToolButton", "data-obj": "homeToolButton" }, [text]);
}

function sectionLabel(text: string) {
  // Qt 里多个 label 同 objectName=homeSectionLabel；Web 里用 data-obj 模拟。
  return el("div", { class: "homeSectionLabel", "data-obj": "homeSectionLabel" }, [text]);
}

function listItem(text: string, onClick: () => void) {
  const li = el("li", { class: "list-item" }, [el("div", {}, [text])]);
  li.addEventListener("click", onClick);
  return li;
}

function formatReminderTitle(r: DeadlineReminder): string {
  const title = (r.contest_title ?? "").trim();
  const dt = (r.deadline_text ?? "").trim();
  if (dt) return `${title} | ${dt}`;
  return title;
}

export function createHomePage(services: AppServices, init: HomeInitial) {
  let toolsExpanded = true;

  const titleLabel = el("div", { id: "homeTitleLabel" }, ["OJ"]);

  const homeBtn = iconButton(services, "homeTopHome", "Home", "homepage.svg");
  const themeBtn = iconButton(services, "homeTopTheme", "Dark Mode", "dark-mode.png");
  const refreshBtn = iconButton(services, "homeTopRefresh", "Refresh", "refresh.svg");

  const topFrame = el("div", { id: "homeTopFrame" }, [
    el("div", { id: "homeTitleBlock" }, [titleLabel]),
    el("div", { id: "homeTopActions" }, [homeBtn, themeBtn, refreshBtn]),
  ]);

  const toolsToggle = el("button", { id: "homeToolsToggleButton" }, ["Tools v"]);

  const favoritesBtn = toolButton("Open Favorites");
  const storageBtn = toolButton("Set");
  const aiConfigBtn = toolButton("AI Config");
  const logoutBtn = toolButton("Log Out");

  const toolsPanel = el("div", { id: "homeToolsPanel" }, [
    favoritesBtn,
    storageBtn,
    aiConfigBtn,
    logoutBtn,
  ]);

  const collapsedFavorites = iconButton(
    services,
    "homeCollapsedFavorites",
    "Favorites",
    "favorite.png"
  );
  const collapsedStorage = iconButton(services, "homeCollapsedStorage", "Set", "set.svg");
  const collapsedAiConfig = iconButton(
    services,
    "homeCollapsedAi",
    "AI Config",
    "ai-config.svg",
    "aiconfig.svg"
  );
  const collapsedLogout = iconButton(
    services,
    "homeCollapsedLogout",
    "Log Out",
    "log out.svg",
    "logout.svg"
  );

  const collapsedPanel = el("div", { id: "homeCollapsedToolsPanel" }, [
    collapsedFavorites,
    collapsedStorage,
    collapsedAiConfig,
    collapsedLogout,
  ]);

  const leftFrame = el("div", { id: "homeLeftFrame" }, [toolsToggle, toolsPanel, collapsedPanel]);

  const classList = el("ul", { id: "homeClassList", class: "list" }, []);
  const middleFrame = el("div", { id: "homeMiddleFrame" }, [sectionLabel("Course List"), classList]);

  const reminderList = el("ul", { id: "homeReminderList", class: "list" }, []);
  const rightFrame = el("div", { id: "homeRightFrame" }, [sectionLabel("Due Soon"), reminderList]);

  const bottom = el("div", { id: "homeBottomRow" }, [leftFrame, middleFrame, rightFrame]);

  const page = el("div", { id: "homePage" }, [topFrame, bottom]);

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

  const renderClasses = (classes: JoinedClassInfo[]) => {
    classList.replaceChildren();
    if (!classes || classes.length === 0) {
      classList.append(el("li", { class: "muted" }, ["No classes."]));
      return;
    }

    for (const c of classes) {
      classList.append(
        listItem(c.name || "Unnamed class", () => {
          services.router.push(createClassPage(services, { name: c.name, url: c.url }));
        })
      );
    }
  };

  const renderReminders = (reminders: DeadlineReminder[]) => {
    reminderList.replaceChildren();
    if (!reminders || reminders.length === 0) {
      reminderList.append(el("li", { class: "muted" }, ["No contest deadlines within one week."]));
      return;
    }

    for (const r of reminders) {
      const text = formatReminderTitle(r);
      const li = el("li", { class: "list-item" }, [el("div", {}, [text])]);
      li.title = `${r.course_name}\n${r.deadline_text}`;
      li.addEventListener("click", () => {
        const contestUrl = String(r.contest_url ?? "");
        if (!contestUrl) return;
        services.router.push(createContestPage(services, { title: r.contest_title, url: contestUrl }));
      });
      reminderList.append(li);
    }
  };

  const showReminderLoading = () => {
    reminderList.replaceChildren(el("li", { class: "muted" }, ["Checking course deadlines..."]));
  };

  const showClassLoading = () => {
    classList.replaceChildren(el("li", { class: "muted" }, ["Loading classes..."]));
  };

  const refreshAll = async () => {
    showClassLoading();
    showReminderLoading();

    try {
      const classes: JoinedClassInfo[] = await services.invoke("oj_get_joined_classes");
      services.setGlobalStatus(true, `classes: ${classes.length}`);
      renderClasses(classes);

      const reminders: DeadlineReminder[] = await services.invoke("oj_due_soon_reminders", { classes });
      services.setGlobalStatus(true, `classes: ${classes.length}, reminders: ${reminders.length}`);
      renderReminders(reminders);
      init.onUpdateReminders(classes, reminders);
    } catch (e) {
      const msg = String(e);
      services.setGlobalStatus(false, `refresh failed: ${msg}`);
      classList.replaceChildren(el("li", { class: "muted err" }, [`Refresh failed: ${msg}`]));
      reminderList.replaceChildren(el("li", { class: "muted err" }, [`Refresh failed: ${msg}`]));
    }
  };

  // events
  toolsToggle.addEventListener("click", () => {
    toolsExpanded = !toolsExpanded;
    applyToolsExpanded();
  });

  refreshBtn.addEventListener("click", () => void refreshAll());
  window.addEventListener("keydown", (ev) => {
    const e = ev as KeyboardEvent;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "r") {
      e.preventDefault();
      void refreshAll();
    }
  });

  themeBtn.addEventListener("click", () => {
    services.setTheme(services.getTheme() === "dark" ? "light" : "dark");
    themeBtn.title = services.getTheme() === "dark" ? "Light Mode" : "Dark Mode";
  });

  homeBtn.addEventListener("click", () => {
    // already home
  });

  const doLogout = async () => {
    try {
      await services.invoke("oj_logout");
    } catch {
      // ignore
    }
    init.onLogout();
  };

  logoutBtn.addEventListener("click", () => void doLogout());
  collapsedLogout.addEventListener("click", () => void doLogout());

  favoritesBtn.addEventListener("click", () => services.router.push(createFavoritePage(services)));
  collapsedFavorites.addEventListener("click", () => services.router.push(createFavoritePage(services)));

  storageBtn.addEventListener("click", () => services.router.push(createStoragePage(services)));
  collapsedStorage.addEventListener("click", () => services.router.push(createStoragePage(services)));

  aiConfigBtn.addEventListener("click", () => services.router.push(createAiConfigPage(services)));
  collapsedAiConfig.addEventListener("click", () => services.router.push(createAiConfigPage(services)));

  applyToolsExpanded();
  renderClasses(init.classes);
  renderReminders(init.reminders);

  void refreshAll();

  return {
    id: "home" as const,
    el: page,
  };
}
