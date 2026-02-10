import { ConvertApp } from "./app/convert-app.js";
import type { AppTauriApi } from "./app/tauri-api.js";
import { getAppElements } from "./app/dom.js";
import { createTauriApi } from "./app/tauri-api.js";
import { toErrorMessage } from "./app/format.js";

function waitForFirstPaint(): Promise<void> {
  return new Promise((resolve) => {
    window.requestAnimationFrame(() => window.requestAnimationFrame(() => resolve()));
  });
}

async function bootstrap(): Promise<void> {
  let tauriApi: AppTauriApi | null = null;

  try {
    tauriApi = createTauriApi();
    const app = new ConvertApp(getAppElements(), tauriApi);
    await app.initialize();
  } catch (error) {
    renderBootstrapError(error);
  } finally {
    if (!tauriApi) {
      return;
    }

    try {
      await waitForFirstPaint();
      await tauriApi.showMainWindow();
    } catch (error) {
      console.error(`Failed to show main window: ${toErrorMessage(error)}`);
    }
  }
}

function renderBootstrapError(error: unknown): void {
  const message = `Failed to initialize app: ${toErrorMessage(error)}`;
  console.error(message);

  const statusElement = document.getElementById("status");
  if (statusElement instanceof HTMLPreElement) {
    statusElement.textContent = message;
  }
}

await bootstrap();
