import type { AppServices, IconName, Theme } from "../app";
import { el, setStatus } from "../dom";
import type { JoinedClassInfo, OpenJudgeLoginResult } from "./types";

export function createLoginPage(
  services: AppServices,
  onLoginSucceeded: (result: OpenJudgeLoginResult) => void | Promise<void>
) {
  const status = el("div", { id: "loginStatusLabel" }, []);

  const emailInput = el("input", {
    id: "loginEmail",
    type: "text",
    placeholder: "name@example.com",
  }) as HTMLInputElement;
  const passwordInput = el("input", {
    id: "loginPassword",
    type: "password",
    placeholder: "Password",
  }) as HTMLInputElement;

  const verifyCodeInput = el("input", {
    id: "loginVerifyCode",
    type: "text",
    placeholder: "Email code",
  }) as HTMLInputElement;

  const verifyButton = el("button", { id: "verifyButton" }, ["Verify"]) as HTMLButtonElement;
  const verifyPanel = el("div", { id: "verifyPanel" }, [
    el("div", { id: "verifyRow" }, [verifyCodeInput, verifyButton]),
  ]);

  const loginButton = el("button", { id: "loginButton" }, ["Login"]) as HTMLButtonElement;

  const card = el("div", { id: "loginCard" }, [
    el("div", { id: "loginTitleLabel" }, ["OpenJudge Login"]),
    el("label", {}, ["Email"]),
    emailInput,
    el("label", {}, ["Password"]),
    passwordInput,
    el("label", { id: "verifyLabel" }, ["Verification Code"]),
    verifyPanel,
    status,
    loginButton,
  ]);

  const shell = el("div", { id: "loginShell" }, [card]);

  const setVerificationVisible = (visible: boolean) => {
    (verifyPanel as HTMLElement).style.display = visible ? "grid" : "none";
    const label = shell.querySelector("#verifyLabel") as HTMLElement | null;
    if (label) label.style.display = visible ? "block" : "none";
    if (!visible) verifyCodeInput.value = "";
    verifyButton.disabled = !visible;
  };

  const setBusy = (busy: boolean) => {
    emailInput.disabled = busy;
    passwordInput.disabled = busy;
    verifyCodeInput.disabled = busy;
    verifyButton.disabled = busy || verifyPanel.style.display === "none";
    loginButton.disabled = busy;
  };

  const checkVerificationRequired = async () => {
    const email = emailInput.value.trim();
    if (!email) {
      setVerificationVisible(false);
      return;
    }

    try {
      const cached: any = await services.invoke("oj_login_cache_lookup", { email });
      if (cached) {
        passwordInput.value = cached.password;
        setVerificationVisible(false);
        return;
      }

      const required: boolean = await services.invoke("oj_requires_email_verification", { email });
      setVerificationVisible(required);
    } catch {
      setVerificationVisible(true);
    }
  };

  const restoreLastLogin = async () => {
    try {
      const last: any = await services.invoke("oj_login_cache_last");
      if (!last) return;
      emailInput.value = last.email;
      passwordInput.value = last.password;
      await checkVerificationRequired();
      setStatus(status, true, "Restored last login.");
    } catch {
      // ignore
    }
  };

  const doSendVerify = async () => {
    const email = emailInput.value.trim();
    if (!email) {
      setStatus(status, false, "Email is required.");
      return;
    }

    try {
      const required: boolean = await services.invoke("oj_requires_email_verification", { email });
      if (!required) {
        setStatus(status, true, "This email is already cached. Verification is not required.");
        return;
      }

      setBusy(true);
      setStatus(status, true, "Sending verification email...");
      const msg: string = await services.invoke("oj_email_send_code", { email });
      setStatus(status, true, msg);
    } catch (e) {
      setStatus(status, false, String(e));
    } finally {
      setBusy(false);
    }
  };

  const doLogin = async () => {
    const email = emailInput.value.trim();
    const password = passwordInput.value;
    if (!email || !password) {
      setStatus(status, false, "Email and password are required.");
      return;
    }

    try {
      const required: boolean = await services.invoke("oj_requires_email_verification", { email });
      if (required) {
        const code = verifyCodeInput.value.trim();
        if (!code) {
          setStatus(status, false, "Verification code is required for a new email.");
          return;
        }
        setBusy(true);
        setStatus(status, true, "Verifying code...");
        await services.invoke("oj_email_verify_code", { email, code });
      }

      setBusy(true);
      setStatus(status, true, "Logging in...");
      const result: OpenJudgeLoginResult = await services.invoke("oj_login", { email, password });

      // DEBUG: show class count from login result
      setStatus(status, true, `Login OK. classes=${result.classes?.length ?? 0}`);

      await onLoginSucceeded(result);
      await checkVerificationRequired();
    } catch (e) {
      setStatus(status, false, String(e));
    } finally {
      setBusy(false);
    }
  };

  emailInput.addEventListener("input", () => {
    void checkVerificationRequired();
  });

  passwordInput.addEventListener("keydown", (ev) => {
    if ((ev as KeyboardEvent).key === "Enter") {
      void doLogin();
    }
  });

  verifyButton.addEventListener("click", () => void doSendVerify());
  loginButton.addEventListener("click", () => void doLogin());

  setStatus(status, true, "");
  setVerificationVisible(false);
  void restoreLastLogin();

  return {
    id: "login" as const,
    el: shell,
  };
}
