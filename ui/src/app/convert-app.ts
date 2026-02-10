import type { AppElements } from "./dom.js";
import { isCancellationErrorMessage, isNoActiveConversionErrorMessage } from "./conversion-errors.js";
import { basename, toErrorMessage } from "./format.js";
import { ProgressPresenter } from "./progress-presenter.js";
import {
  deduplicatePaths,
  formatFileCount,
  formatSelectedInputLabel,
  remainingQueueCount
} from "./queue-utils.js";
import type { AppTauriApi } from "./tauri-api.js";
import type { ConvertProgressPayload, ConvertRequest } from "./types.js";
import { UiState } from "./ui-state.js";

type EncoderStatusTone = "info" | "ok" | "warn";

export class ConvertApp {
  private readonly uiState: UiState;
  private readonly progressPresenter: ProgressPresenter;
  private isConverting = false;
  private cancelRequested = false;
  private queuedInputPaths: string[] = [];
  private hasAvailableEncoder = false;
  private isLoadingEncoder = false;
  private encoderLoadPromise: Promise<void> | null = null;
  private selectedAv1Encoder: string | null = null;
  private isQueueItemRunning = false;
  private detachProgressListener: (() => void | Promise<void>) | null = null;

  constructor(
    private readonly elements: AppElements,
    private readonly tauriApi: AppTauriApi
  ) {
    this.uiState = new UiState(elements);
    this.progressPresenter = new ProgressPresenter(this.uiState);
  }

  async initialize(): Promise<void> {
    this.registerEventHandlers();
    void this.attachProgressListener();
    this.setEncoderStatus("Detecting AV1 encoder...", "info");
    void this.ensureEncodersLoaded(false);
    this.uiState.appendStatus("Ready.");
    this.syncButtonState();
  }

  private registerEventHandlers(): void {
    this.elements.browseInputButton.addEventListener("click", () => void this.handleBrowseInput());
    this.elements.convertButton.addEventListener("click", () => void this.handleConvert());
    this.elements.cancelButton.addEventListener("click", () => void this.handleCancel());
    window.addEventListener("beforeunload", () => {
      this.teardownProgressListener();
    });
  }

  private async attachProgressListener(): Promise<void> {
    try {
      const unlisten = await this.tauriApi.listenConvertProgress((payload) =>
        this.handleProgress(payload)
      );
      this.detachProgressListener = () => {
        void unlisten();
        this.detachProgressListener = null;
      };
    } catch (error) {
      this.uiState.appendStatus(`Progress listener unavailable: ${toErrorMessage(error)}`);
    }
  }

  private teardownProgressListener(): void {
    if (!this.detachProgressListener) {
      return;
    }

    void this.detachProgressListener();
  }

  private async ensureEncodersLoaded(reportError: boolean): Promise<void> {
    if (this.hasAvailableEncoder || this.isLoadingEncoder) {
      return this.encoderLoadPromise ?? Promise.resolve();
    }

    this.encoderLoadPromise = this.loadEncoders(reportError).finally(() => {
      this.encoderLoadPromise = null;
    });
    return this.encoderLoadPromise;
  }

  private async loadEncoders(reportError: boolean): Promise<void> {
    this.isLoadingEncoder = true;
    this.setEncoderStatus("Detecting AV1 encoder...", "info");
    this.syncButtonState();

    try {
      const selectedEncoder = await this.tauriApi.pickAutoAv1Encoder();
      this.selectedAv1Encoder = selectedEncoder;
      this.hasAvailableEncoder = true;
      this.setEncoderStatus(`AV1 encoder ready: ${this.selectedAv1Encoder}`, "ok");
      this.uiState.appendStatus(`Encoder auto-selected: ${this.selectedAv1Encoder}`);
    } catch (error) {
      this.setEncoderStatus("No AV1 encoder available. Check ffmpeg/encoder setup.", "warn");
      if (reportError) {
        this.uiState.appendStatus(`Failed to load encoders: ${toErrorMessage(error)}`);
      }
      this.selectedAv1Encoder = null;
      this.hasAvailableEncoder = false;
    } finally {
      this.isLoadingEncoder = false;
      this.syncButtonState();
    }
  }

  private async handleBrowseInput(): Promise<void> {
    if (this.isConverting) {
      return;
    }

    try {
      const inputPaths = await this.tauriApi.openInputDialog();
      if (inputPaths.length === 0) {
        return;
      }

      const normalizedInputPaths = deduplicatePaths(inputPaths);
      this.queuedInputPaths = normalizedInputPaths;
      this.elements.inputPath.value = formatSelectedInputLabel(normalizedInputPaths);
      this.syncButtonState();
      this.uiState.appendStatus(`Queued ${formatFileCount(normalizedInputPaths.length)} for conversion.`);
      void this.ensureEncodersLoaded(false);
    } catch (error) {
      this.uiState.appendStatus(`Browse input failed: ${toErrorMessage(error)}`);
    }
  }

  private async handleConvert(): Promise<void> {
    if (this.isConverting) {
      return;
    }

    await this.ensureEncodersLoaded(true);

    if (this.queuedInputPaths.length === 0) {
      this.uiState.appendStatus("Select at least one input file.");
      return;
    }

    if (!this.hasAvailableEncoder) {
      this.uiState.appendStatus("No AV1 encoder available.");
      return;
    }

    this.uiState.clearStatus();
    this.uiState.setProgress(0);
    this.progressPresenter.reset();
    this.cancelRequested = false;
    this.isQueueItemRunning = false;
    this.setConverting(true);

    const queueLength = this.queuedInputPaths.length;
    this.uiState.appendStatus(`Queue length: ${formatFileCount(queueLength)}`);
    this.uiState.appendStatus(
      this.selectedAv1Encoder
        ? `Encoder mode: Auto (selected: ${this.selectedAv1Encoder})`
        : "Encoder mode: Auto"
    );
    this.uiState.appendStatus("Starting queued conversions...");

    const queue = [...this.queuedInputPaths];
    try {
      let successCount = 0;
      let failureCount = 0;

      for (const [index, inputPath] of queue.entries()) {
        if (this.cancelRequested) {
          break;
        }

        this.uiState.setProgress(0, `Queue ${index + 1}/${queue.length} | Waiting`);
        this.progressPresenter.reset();

        this.uiState.appendStatus(`Queue ${index + 1}/${queue.length}: ${basename(inputPath)}`);

        const request = this.createConvertRequest(inputPath);

        try {
          const result = await this.tauriApi.convertVideo(request);
          if (this.cancelRequested) {
            break;
          }

          successCount += 1;
          this.uiState.setProgress(100, `Queue ${index + 1}/${queue.length} | Done`);
          this.uiState.appendStatus(`Done: ${result.outputPath}`);
          this.uiState.appendStatus(`Encoder used: ${result.av1Encoder}`);
          this.uiState.appendStatus(`Target size: ${result.targetSizeBytes} bytes`);
          this.uiState.appendStatus(`Video bitrate: ${result.videoBitrateKbps} kbps`);
          this.uiState.appendStatus(`Audio bitrate: ${result.audioBitrateKbps} kbps`);
        } catch (error) {
          const message = toErrorMessage(error);
          if (isCancellationErrorMessage(message) || this.cancelRequested) {
            this.cancelRequested = true;
            this.uiState.appendStatus("Canceled by user.");
            break;
          }

          failureCount += 1;
          this.uiState.appendStatus(`Failed: ${message}`);
        } finally {
          this.isQueueItemRunning = false;
          this.syncButtonState();
        }
      }

      if (this.cancelRequested) {
        const remainingCount = remainingQueueCount(queue.length, successCount, failureCount);
        this.uiState.appendStatus(
          `Queue canceled. Completed: ${successCount}, Failed: ${failureCount}, Remaining: ${remainingCount}`
        );
      } else {
        this.uiState.appendStatus(`Queue complete. Succeeded: ${successCount}, Failed: ${failureCount}`);
      }
    } finally {
      if (this.cancelRequested) {
        this.progressPresenter.reset();
        this.uiState.setProgress(0, "Canceled");
      } else {
        this.progressPresenter.flushNow();
      }
      this.setConverting(false);
    }
  }

  private async handleCancel(): Promise<void> {
    if (!this.isConverting || this.cancelRequested) {
      return;
    }

    this.cancelRequested = true;
    this.isQueueItemRunning = false;
    this.progressPresenter.reset();
    this.uiState.setProgress(0, "Canceling...");
    this.syncButtonState();
    this.uiState.appendStatus("Cancel requested...");

    try {
      await this.tauriApi.cancelConversion();
    } catch (error) {
      const message = toErrorMessage(error);
      if (!isNoActiveConversionErrorMessage(message)) {
        this.uiState.appendStatus(`Cancel request failed: ${message}`);
      }
    }
  }

  private handleProgress(payload: ConvertProgressPayload): void {
    if (!this.isConverting || this.cancelRequested) {
      return;
    }

    if (!this.isQueueItemRunning) {
      this.isQueueItemRunning = true;
      this.syncButtonState();
    }

    this.progressPresenter.queue(payload);
  }

  private setConverting(value: boolean): void {
    if (!value) {
      this.cancelRequested = false;
      this.isQueueItemRunning = false;
    }

    this.isConverting = value;
    this.elements.appRoot.setAttribute("aria-busy", value ? "true" : "false");
    this.elements.appRoot.classList.toggle("is-converting", value);
    this.syncButtonState();
  }

  private syncButtonState(): void {
    const hasSelectedInput = this.queuedInputPaths.length > 0;

    this.elements.browseInputButton.disabled = this.isConverting;
    this.elements.convertButton.disabled =
      this.isConverting ||
      this.isLoadingEncoder ||
      !this.hasAvailableEncoder ||
      !hasSelectedInput;
    this.elements.convertButton.setAttribute(
      "aria-disabled",
      this.elements.convertButton.disabled ? "true" : "false"
    );
    this.elements.cancelButton.hidden = !this.isConverting;
    this.elements.cancelButton.disabled = !this.isConverting || this.cancelRequested;
  }

  private createConvertRequest(inputPath: string): ConvertRequest {
    const request: ConvertRequest = { inputPath };
    if (this.selectedAv1Encoder) {
      request.av1Encoder = this.selectedAv1Encoder;
    }
    return request;
  }

  private setEncoderStatus(message: string, tone: EncoderStatusTone): void {
    this.elements.encoderStatus.textContent = message;
    this.elements.encoderStatus.classList.remove("ok", "warn");
    if (tone === "ok") {
      this.elements.encoderStatus.classList.add("ok");
      return;
    }
    if (tone === "warn") {
      this.elements.encoderStatus.classList.add("warn");
    }
  }
}
