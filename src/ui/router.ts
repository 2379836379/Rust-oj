export type PageId =
  | "login"
  | "home"
  | "class"
  | "contest"
  | "problem"
  | "favorite"
  | "storage"
  | "aiconfig";

export type Page = {
  id: PageId;
  el: HTMLElement;
  onShow?: () => void | Promise<void>;
  onHide?: () => void;
};

export class StackRouter {
  private stack: Page[] = [];

  constructor(private root: HTMLElement) {}

  current(): Page | null {
    return this.stack.length ? this.stack[this.stack.length - 1] : null;
  }

  reset(page: Page) {
    for (const p of this.stack) p.onHide?.();
    this.stack = [page];
    this.root.replaceChildren(page.el);
    void page.onShow?.();
  }

  push(page: Page) {
    const cur = this.current();
    cur?.onHide?.();
    this.stack.push(page);
    this.root.replaceChildren(page.el);
    void page.onShow?.();
  }

  pop() {
    if (this.stack.length <= 1) return;
    const cur = this.stack.pop();
    cur?.onHide?.();
    const next = this.current();
    if (!next) return;
    this.root.replaceChildren(next.el);
    void next.onShow?.();
  }
}