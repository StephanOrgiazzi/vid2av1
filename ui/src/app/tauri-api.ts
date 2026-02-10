import type {
  ConvertProgressPayload,
  ConvertRequest,
  ConvertResult,
  DialogFilter,
  DialogOpenOptions,
  TauriDialogApi,
  TauriGlobalApi
} from "./types.js";

const VIDEO_FILE_FILTERS: DialogFilter[] = [
  { name: "Video files", extensions: ["mp4", "mkv", "mov", "avi", "webm", "m4v", "ts"] }
];

export interface AppTauriApi {
  pickAutoAv1Encoder(): Promise<string>;
  convertVideo(request: ConvertRequest): Promise<ConvertResult>;
  cancelConversion(): Promise<void>;
  showMainWindow(): Promise<void>;
  openInputDialog(): Promise<string[]>;
  listenConvertProgress(
    listener: (payload: ConvertProgressPayload) => void
  ): Promise<() => void | Promise<void>>;
}

export function createTauriApi(tauriGlobal: TauriGlobalApi | undefined = window.__TAURI__): AppTauriApi {
  if (!tauriGlobal) {
    throw new Error("window.__TAURI__ is unavailable. Enable app.withGlobalTauri in tauri.conf.json.");
  }

  const invoke = tauriGlobal.core.invoke.bind(tauriGlobal.core);
  const listen = tauriGlobal.event.listen.bind(tauriGlobal.event);
  const invokeDialogOpen = (options: DialogOpenOptions) =>
    invoke<unknown>("plugin:dialog|open", { options });

  return {
    pickAutoAv1Encoder: () => invoke<string>("pick_auto_av1_encoder"),

    convertVideo: (request: ConvertRequest) => invoke<ConvertResult>("convert_video", { request }),

    cancelConversion: () => invoke<void>("cancel_conversion"),

    showMainWindow: () => invoke<void>("show_main_window"),

    openInputDialog: () => {
      const options = { multiple: true, filters: VIDEO_FILE_FILTERS };
      return openInputDialogWithFallback(tauriGlobal.dialog, options, invokeDialogOpen);
    },

    listenConvertProgress(
      listener: (payload: ConvertProgressPayload) => void
    ): Promise<() => void | Promise<void>> {
      return listen("convert-progress", (event: { payload: ConvertProgressPayload }) => {
        listener(event.payload ?? {});
      });
    }
  };
}

async function openInputDialogWithFallback(
  dialogApi: TauriDialogApi | undefined,
  options: DialogOpenOptions,
  invokeDialogOpen: (options: DialogOpenOptions) => Promise<unknown>
): Promise<string[]> {
  const openViaInvoke = async (): Promise<string[]> => {
    const selected = await invokeDialogOpen(options);
    return normalizeSelectedPaths(selected);
  };

  if (dialogApi && typeof dialogApi.open === "function") {
    try {
      const selected = await dialogApi.open(options);
      return normalizeSelectedPaths(selected);
    } catch {
      return openViaInvoke();
    }
  }

  return openViaInvoke();
}

function normalizeSelectedPaths(selected: unknown): string[] {
  if (typeof selected === "string") {
    return selected.length > 0 ? [selected] : [];
  }

  if (Array.isArray(selected)) {
    return selected.filter((entry): entry is string => typeof entry === "string" && entry.length > 0);
  }

  return [];
}
