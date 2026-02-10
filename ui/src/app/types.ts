export interface ConvertRequest {
  inputPath: string;
  av1Encoder?: string;
}

export interface ConvertResult {
  outputPath: string;
  targetSizeBytes: number;
  videoBitrateKbps: number;
  audioBitrateKbps: number;
  av1Encoder: string;
}

export interface ConvertProgressPayload {
  percent?: number;
  label?: string;
  speed?: number;
  etaSeconds?: number;
}

export interface DialogFilter {
  name: string;
  extensions: string[];
}

export interface DialogOpenOptions {
  multiple: boolean;
  filters: DialogFilter[];
}

export interface TauriDialogApi {
  open(options: DialogOpenOptions): Promise<string | string[] | null>;
}

export interface TauriCoreApi {
  invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}

export interface TauriEventApi {
  listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void | Promise<void>>;
}

export interface TauriGlobalApi {
  core: TauriCoreApi;
  event: TauriEventApi;
  dialog?: TauriDialogApi;
}

declare global {
  interface Window {
    __TAURI__?: TauriGlobalApi;
  }
}

export {};
