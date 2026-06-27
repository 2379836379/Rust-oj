import type { AppServices, IconName } from "../app";
import { el } from "../dom";
import type { FavoriteFolderInfo, FavoriteProblemRow, ProblemPageInfo } from "./types";

import { createProblemPage } from "./problem";

type FavoriteInit = {
  // reserved
};

type ViewMode = "folders" | "favorites";

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

export function createFavoritePage(services: AppServices, _init?: FavoriteInit) {
  let toolsExpanded = true;
  let mode: ViewMode = "folders";

  let currentFolder: FavoriteFolderInfo | null = null;
  let selectedFolderId: number | null = null;
  let selectedProblemUrl: string | null = null;

  const titleLabel = el("div", { id: "favoriteTitleLabel" }, ["Favorites"]);

  const homeBtn = iconButton(services, "favoriteTopHome", "Home", "homepage.svg");
  const themeBtn = iconButton(services, "favoriteTopTheme", "Dark Mode", "dark-mode.png");
  const refreshBtn = iconButton(services, "favoriteTopRefresh", "Refresh", "refresh.svg");

  const topFrame = el("div", { id: "favoriteTopFrame" }, [
    titleLabel,
    el("div", { id: "favoriteTopActions" }, [homeBtn, themeBtn, refreshBtn]),
  ]);

  const toolsToggle = el("button", { id: "favoriteToolsToggleButton" }, ["Tools v"]);
  const backBtn = el("button", { id: "favoriteToolButton" }, ["Back"]);
  const toolsPanel = el("div", { id: "favoriteToolsPanel" }, [backBtn]);

  const collapsedBack = iconButton(services, "favoriteCollapsedBack", "Back", "back.svg");
  const collapsedPanel = el("div", { id: "favoriteCollapsedToolsPanel" }, [collapsedBack]);

  const leftFrame = el("div", { id: "favoriteLeftFrame" }, [toolsToggle, toolsPanel, collapsedPanel]);

  const sectionLabel = el("div", { id: "favoriteSectionLabel" }, ["Favorite Folders"]);
  const statusLabel = el("div", { id: "favoriteStatusLabel", class: "muted" }, [""]);

  const list = el("ul", { id: "favoriteListWidget", class: "list" }, []);
  const actionButton = (text: string) =>
    el("button", { class: "favoriteActionButton", "data-obj": "favoriteActionButton" }, [text]);

  const newFolderBtn = actionButton("New Folder");
  const deleteFolderBtn = actionButton("Delete Folder");
  const removeSelectedBtn = actionButton("Remove Selected");

  deleteFolderBtn.setAttribute("disabled", "true");
  removeSelectedBtn.style.display = "none";
  removeSelectedBtn.setAttribute("disabled", "true");

  const folderActions = el("div", { class: "qtActionRow" }, [newFolderBtn, deleteFolderBtn]);

  const contentFrame = el("div", { id: "favoriteContentFrame" }, [
    sectionLabel,
    statusLabel,
    list,
    folderActions,
    removeSelectedBtn,
  ]);

  const bottom = el("div", { id: "favoriteBottomRow" }, [leftFrame, contentFrame]);
  const page = el("div", { id: "favoritePage" }, [topFrame, bottom]);

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

  const setViewMode = (m: ViewMode, folderName?: string) => {
    mode = m;
    selectedProblemUrl = null;

    if (mode === "folders") {
      sectionLabel.textContent = "Favorite Folders";
      newFolderBtn.style.display = "inline-flex";
      deleteFolderBtn.style.display = "inline-flex";
      folderActions.style.display = "flex";
      removeSelectedBtn.style.display = "none";
      removeSelectedBtn.setAttribute("disabled", "true");
    } else {
      sectionLabel.textContent = folderName ? `Favorite Problems / ${folderName}` : "Favorite Problems";
      folderActions.style.display = "none";
      removeSelectedBtn.style.display = "inline-flex";
      removeSelectedBtn.setAttribute("disabled", "true");
    }

    statusLabel.textContent = "";
  };

  const clearSelection = () => {
    for (const li of Array.from(list.querySelectorAll(".list-item"))) {
      li.classList.remove("selected");
    }
  };

  const renderFolders = (folders: FavoriteFolderInfo[]) => {
    list.replaceChildren();
    selectedFolderId = null;
    deleteFolderBtn.setAttribute("disabled", "true");

    if (!folders.length) {
      statusLabel.textContent = "No favorite folders yet.";
      return;
    }

    for (const f of folders) {
      const li = el("li", { class: "list-item" }, [el("div", {}, [`${f.name} (${f.item_count})`])]);
      li.addEventListener("click", async () => {
        clearSelection();
        li.classList.add("selected");
        selectedFolderId = f.id;
        deleteFolderBtn.removeAttribute("disabled");

        // open folder
        currentFolder = f;
        await loadFavorites(f.id, f.name);
      });
      list.append(li);
    }
  };

  const renderFavorites = (items: FavoriteProblemRow[], folderName: string) => {
    list.replaceChildren();
    clearSelection();
    selectedProblemUrl = null;
    removeSelectedBtn.setAttribute("disabled", "true");

    if (!items.length) {
      statusLabel.textContent = "This folder has no favorite problems.";
      return;
    }

    for (const it of items) {
      const title = it.title?.trim() ? it.title : it.problem_url;
      const li = el("li", { class: "list-item" }, [
        el("div", {}, [title]),
        el("div", { class: "muted" }, [it.saved_at]),
      ]);

      li.addEventListener("click", async () => {
        clearSelection();
        li.classList.add("selected");
        selectedProblemUrl = it.problem_url;
        removeSelectedBtn.removeAttribute("disabled");

        // open problem from db
        try {
          const p: ProblemPageInfo | null = await services.invoke("oj_favorite_load_problem", {
            problem_url: it.problem_url,
          });
          if (!p) {
            services.setGlobalStatus(false, "Favorite problem not found in DB.");
            return;
          }
          services.router.push(createProblemPage(services, { info: p, contestTitle: folderName }));
        } catch (e) {
          services.setGlobalStatus(false, String(e));
        }
      });

      list.append(li);
    }
  };

  const loadFolders = async () => {
    setViewMode("folders");
    list.replaceChildren(el("li", { class: "muted" }, ["Loading folders..."]));

    try {
      const folders: FavoriteFolderInfo[] = await services.invoke("oj_favorite_list_folders");
      renderFolders(folders);
      services.setGlobalStatus(true, "");
    } catch (e) {
      statusLabel.textContent = String(e);
      list.replaceChildren();
      services.setGlobalStatus(false, String(e));
    }
  };

  const loadFavorites = async (folderId: number, folderName: string) => {
    setViewMode("favorites", folderName);
    list.replaceChildren(el("li", { class: "muted" }, ["Loading favorites..."]));

    try {
      const items: FavoriteProblemRow[] = await services.invoke("oj_favorite_list_folder_items", {
        folder_id: folderId,
      });
      renderFavorites(items, folderName);
      services.setGlobalStatus(true, "");
    } catch (e) {
      statusLabel.textContent = String(e);
      list.replaceChildren();
      services.setGlobalStatus(false, String(e));
    }
  };

  const refresh = async () => {
    if (mode === "folders") {
      await loadFolders();
      statusLabel.textContent = "Favorite folders refreshed.";
      return;
    }

    if (currentFolder) {
      await loadFavorites(currentFolder.id, currentFolder.name);
      statusLabel.textContent = "Favorite list refreshed.";
    }
  };

  const createFolder = async () => {
    const name = (window.prompt("New folder name") ?? "").trim();
    if (!name) return;

    try {
      await services.invoke("oj_favorite_create_folder", { name });
      await loadFolders();
    } catch (e) {
      statusLabel.textContent = String(e);
      services.setGlobalStatus(false, String(e));
    }
  };

  const deleteFolder = async () => {
    if (!selectedFolderId) return;
    const ok = window.confirm("Delete selected folder?");
    if (!ok) return;

    try {
      await services.invoke("oj_favorite_delete_folder", { folder_id: selectedFolderId });
      await loadFolders();
    } catch (e) {
      statusLabel.textContent = String(e);
      services.setGlobalStatus(false, String(e));
    }
  };

  const removeSelected = async () => {
    if (!currentFolder || !selectedProblemUrl) return;

    try {
      await services.invoke("oj_favorite_remove_item", {
        folder_id: currentFolder.id,
        problem_url: selectedProblemUrl,
      });
      await loadFavorites(currentFolder.id, currentFolder.name);
    } catch (e) {
      statusLabel.textContent = String(e);
      services.setGlobalStatus(false, String(e));
    }
  };

  // events
  toolsToggle.addEventListener("click", () => {
    toolsExpanded = !toolsExpanded;
    applyToolsExpanded();
  });

  backBtn.addEventListener("click", () => {
    if (mode === "favorites") {
      currentFolder = null;
      void loadFolders();
      return;
    }
    services.router.pop();
  });
  collapsedBack.addEventListener("click", () => (backBtn as HTMLButtonElement).click());

  newFolderBtn.addEventListener("click", () => void createFolder());
  deleteFolderBtn.addEventListener("click", () => void deleteFolder());
  removeSelectedBtn.addEventListener("click", () => void removeSelected());

  refreshBtn.addEventListener("click", () => void refresh());

  themeBtn.addEventListener("click", () => {
    services.setTheme(services.getTheme() === "dark" ? "light" : "dark");
    themeBtn.title = services.getTheme() === "dark" ? "Light Mode" : "Dark Mode";
  });

  homeBtn.addEventListener("click", () => services.goHome());

  applyToolsExpanded();

  return {
    id: "favorite" as const,
    el: page,
    onShow: () => loadFolders(),
  };
}