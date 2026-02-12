import { toDisplayErrorMessage } from "./conversion-errors.js";
import type { ConvertProgressPayload } from "./types.js";

export function toErrorMessage(error: unknown): string {
  return toDisplayErrorMessage(toRawErrorMessage(error));
}

export function toRawErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function basename(filePath: string): string {
  const normalizedPath = filePath.replace(/\\/g, "/");
  const lastSeparatorIndex = normalizedPath.lastIndexOf("/");
  return lastSeparatorIndex >= 0 ? normalizedPath.slice(lastSeparatorIndex + 1) : normalizedPath;
}

export function clampPercent(value: number): number {
  return Math.min(100, Math.max(0, value));
}

export function formatPercent(value: number): string {
  return `${clampPercent(value).toFixed(1)}%`;
}

export function formatProgressText(payload: ConvertProgressPayload): string {
  const percent = payload.percent ?? 0;
  const parts: string[] = [];

  if (payload.label) {
    parts.push(payload.label);
  }

  parts.push(formatPercent(percent));

  if (typeof payload.speed === "number" && payload.speed > 0) {
    parts.push(`${payload.speed.toFixed(2)}x`);
  }

  if (typeof payload.etaSeconds === "number" && payload.etaSeconds >= 0) {
    parts.push(`ETA ${Math.round(payload.etaSeconds)}s`);
  }

  return parts.join(" | ");
}
