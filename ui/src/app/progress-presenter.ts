import { formatProgressText } from "./format.js";
import type { ConvertProgressPayload } from "./types.js";
import { UiState } from "./ui-state.js";

const PERCENT_RENDER_THRESHOLD = 0.1;

export class ProgressPresenter {
  private pendingPayload: ConvertProgressPayload | null = null;
  private progressRafId: number | null = null;
  private lastRenderedProgressText = "";
  private lastRenderedPercent = -1;

  constructor(private readonly uiState: UiState) {}

  queue(payload: ConvertProgressPayload): void {
    this.pendingPayload = payload;
    if (this.progressRafId !== null) {
      return;
    }

    this.progressRafId = window.requestAnimationFrame(() => {
      this.progressRafId = null;
      this.flushNow();
    });
  }

  flushNow(): void {
    if (!this.pendingPayload) {
      return;
    }

    const payload = this.pendingPayload;
    this.pendingPayload = null;

    const percent = payload.percent ?? 0;
    const progressText = formatProgressText(payload);
    const percentChanged = Math.abs(percent - this.lastRenderedPercent) >= PERCENT_RENDER_THRESHOLD;
    const textChanged = progressText !== this.lastRenderedProgressText;
    if (!percentChanged && !textChanged) {
      return;
    }

    this.uiState.setProgress(percent, progressText);
    this.lastRenderedPercent = percent;
    this.lastRenderedProgressText = progressText;
  }

  reset(): void {
    if (this.progressRafId !== null) {
      window.cancelAnimationFrame(this.progressRafId);
      this.progressRafId = null;
    }

    this.pendingPayload = null;
    this.lastRenderedPercent = -1;
    this.lastRenderedProgressText = "";
  }
}
