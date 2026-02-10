export function deduplicatePaths(inputPaths: string[]): string[] {
  return Array.from(new Set(inputPaths));
}

export function formatFileCount(count: number): string {
  return `${count} file${count === 1 ? "" : "s"}`;
}

export function formatSelectedInputLabel(inputPaths: string[]): string {
  if (inputPaths.length === 1) {
    return inputPaths[0] ?? "";
  }

  return `${formatFileCount(inputPaths.length)} selected`;
}

export function remainingQueueCount(
  totalItems: number,
  succeededItems: number,
  failedItems: number
): number {
  return Math.max(0, totalItems - succeededItems - failedItems);
}
