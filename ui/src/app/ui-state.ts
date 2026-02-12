import type { AppElements } from "./dom.js";
import { clampPercent, formatPercent } from "./format.js";

const MAX_STATUS_LINES = 400;

export class UiState {
  private readonly statusLineNodes: Text[] = [];

  constructor(private readonly elements: Pick<AppElements, "progress" | "progressText" | "status">) {}

  appendStatus(line: string): void {
    const lineNode = document.createTextNode(`${line}\n`);
    this.elements.status.appendChild(lineNode);
    this.statusLineNodes.push(lineNode);

    while (this.statusLineNodes.length > MAX_STATUS_LINES) {
      const removed = this.statusLineNodes.shift();
      removed?.remove();
    }

    this.elements.status.scrollTop = this.elements.status.scrollHeight;
  }

  clearStatus(): void {
    this.statusLineNodes.length = 0;
    this.elements.status.textContent = "";
  }

  setProgress(percent: number, textOverride?: string): void {
    const clampedPercent = clampPercent(percent);
    this.elements.progress.value = clampedPercent;
    this.elements.progressText.textContent = textOverride ?? formatPercent(clampedPercent);
  }
}
