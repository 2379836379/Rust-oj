import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { StackRouter } from "./router";
import { setStatus } from "./dom";
import type {
  AlarmTrigger,
  DeadlineReminder,
  JoinedClassInfo,
  OpenJudgeLoginResult,
} from "./pages/types";

import { createLoginPage } from "./pages/login";
import { createHomePage } from "./pages/home";

export type Theme = "light" | "dark";

export type IconName =
  | "homepage.svg"
  | "dark-mode.png"
  | "light-mode.svg"
  | "refresh.svg"
  | "favorite.png"
  | "set.svg"
  | "ai-config.svg"
  | "aiconfig.svg"
  | "log out.svg"
  | "logout.svg"
  | "back.svg";

export type AppServices = {
  invoke: typeof invoke;
  router: StackRouter;
  getTheme: () => Theme;
  setTheme: (theme: Theme) => void;
  iconUrl: (name: IconName) => string;
  registerIcon: (img: HTMLImageElement, light: IconName, dark?: IconName) => void;
  setGlobalStatus: (ok: boolean, message: string) => void;
  setReminderState: (classes: JoinedClassInfo[], reminders: DeadlineReminder[]) => void;
  getReminderState: () => { classes: JoinedClassInfo[]; reminders: DeadlineReminder[] };
  goHome: () => void;
};

function computeNextHourDelayMs(now: Date) {
  const next = new Date(now);
  next.setMinutes(0, 0, 0);
  next.setHours(now.getHours() + 1);
  return Math.max(1000, next.getTime() - now.getTime());
}

function scheduleHourly(fn: () => void | Promise<void>) {
  const tick = () => {
    Promise.resolve()
      .then(() => fn())
      .catch(() => {
        // ignore
      })
      .finally(() => {
        window.setTimeout(tick, computeNextHourDelayMs(new Date()));
      });
  };
  window.setTimeout(tick, computeNextHourDelayMs(new Date()));
}

export function mountApp(root: HTMLElement) {
  if (!root) throw new Error("missing #app");

  root.replaceChildren();

  const shell = document.createElement("div");
  shell.id = "appShell";

  const stage = document.createElement("div");
  stage.id = "pageStage";

  const globalStatus = document.createElement("div");
  globalStatus.id = "appGlobalStatus";
  globalStatus.className = "muted";

  shell.append(stage, globalStatus);
  root.append(shell);

  const router = new StackRouter(stage);

  let theme: Theme = localStorage.getItem("theme") === "dark" ? "dark" : "light";
  const applyTheme = () => {
    document.documentElement.dataset.theme = theme;
  };
  applyTheme();

  const iconsUsed: HTMLImageElement[] = [];
  const normalizeIconName = (name: IconName, t: Theme): IconName => {
    if (t !== "dark") return name;
    if (name === "dark-mode.png") return "light-mode.svg";
    if (name === "ai-config.svg") return "aiconfig.svg";
    if (name === "log out.svg") return "logout.svg";
    return name;
  };

  const iconUrl = (name: IconName) =>
    encodeURI(`/images/${theme === "dark" ? "dark_mode" : "light_mode"}/${normalizeIconName(name, theme)}`);

  const registerIcon = (img: HTMLImageElement, light: IconName, dark?: IconName) => {
    img.dataset.iconLight = light;
    img.dataset.iconDark = dark ?? light;
    iconsUsed.push(img);
    img.src = iconUrl(theme === "dark" ? (img.dataset.iconDark as IconName) : light);
  };

  const refreshIcons = () => {
    for (const img of iconsUsed) {
      const light = (img.dataset.iconLight ?? "refresh.svg") as IconName;
      const dark = (img.dataset.iconDark ?? light) as IconName;
      img.src = iconUrl(theme === "dark" ? dark : light);
    }
  };

  const setTheme = (t: Theme) => {
    theme = t;
    localStorage.setItem("theme", theme);
    applyTheme();
    refreshIcons();
  };

  let reminderClasses: JoinedClassInfo[] = [];
  let reminderList: DeadlineReminder[] = [];

  const goHome = () => {
    const page = createHomePage(services, {
      classes: reminderClasses,
      reminders: reminderList,
      onUpdateReminders: (classes, reminders) => services.setReminderState(classes, reminders),
      onLogout: () => showLogin(),
      onOpenClass: (name, url) => {
        // wired in HomePage when class page is implemented.
      },
    });
    router.reset(page);
  };

  const services: AppServices = {
    invoke,
    router,
    getTheme: () => theme,
    setTheme,
    iconUrl,
    registerIcon,
    setGlobalStatus: (ok, msg) => setStatus(globalStatus, ok, msg),
    setReminderState: (classes, reminders) => {
      reminderClasses = classes;
      reminderList = reminders;
    },
    getReminderState: () => ({ classes: reminderClasses, reminders: reminderList }),
    goHome,
  };

  // system tray first-close notice (Qt behavior)
  void listen("oj_tray_first_close", async () => {
    try {
      await Notification.requestPermission();
      new Notification("oj-client", { body: "Minimized to tray." });
    } catch {
      // ignore
    }
  });

  // hourly alarm check: align to next full hour.
  scheduleHourly(async () => {
    const st: any = await invoke("get_app_state");
    if (!st?.alarm_enabled) return;

    const { reminders } = services.getReminderState();
    if (!reminders.length) return;

    const triggers: AlarmTrigger[] = await invoke("oj_alarm_process_reminders", { reminders });
    for (const t of triggers) {
      const r = t.reminder;
      const text =
        (r.course_name ?? "") +
        "\n" +
        (r.contest_title ?? "") +
        "\nDeadline: " +
        (r.deadline_text ?? "") +
        "\n" +
        String(t.hours_before) +
        " hour(s) left";

      try {
        await Notification.requestPermission();
        new Notification("Contest deadline alarm", { body: text });
      } catch {
        // ignore
      }

      try {
        const ring = String(st.ring_path ?? "").trim();
        if (ring) {
          const normalized = ring.replace(/\\/g, "/");
          const url = normalized.startsWith("/") ? "file://" + normalized : "file:///" + normalized;
          await new Audio(url).play();
        }
      } catch {
        // ignore
      }
    }
  });

  const showLogin = () => {
    const page = createLoginPage(services, async (result: OpenJudgeLoginResult) => {
      reminderClasses = result.classes ?? [];
      reminderList = [];
      services.setGlobalStatus(true, "Login succeeded.");
      goHome();
    });
    router.reset(page);
  };

  showLogin();
}

mountApp(document.getElementById("app") as HTMLElement);
