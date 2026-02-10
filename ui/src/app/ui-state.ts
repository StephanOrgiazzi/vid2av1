import type { AppElements } from "./dom.js";
import { clampPercent, formatPercent } from "./format.js";

export class UiState {
  constructor(private readonly elements: Pick<AppElements, "progress" | "progressText" | "status">) {}

  appendStatus(line: string): void {
    this.elements.status.textContent += `${line}\n`;
    this.elements.status.scrollTop = this.elements.status.scrollHeight;
  }

  clearStatus(): void {
    this.elements.status.textContent = "";
  }

  setProgress(percent: number, textOverride?: string): void {
    const clampedPercent = clampPercent(percent);
    this.elements.progress.value = clampedPercent;
    this.elements.progressText.textContent = textOverride ?? formatPercent(clampedPercent);
  }
}
