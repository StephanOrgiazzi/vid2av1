function requireElement<T extends HTMLElement>(id: string, expected: typeof HTMLElement): T {
  const element = document.getElementById(id);
  if (!(element instanceof expected)) {
    throw new Error(`Expected element "${id}" to be a ${expected.name}.`);
  }
  return element as T;
}

export interface AppElements {
  appRoot: HTMLElement;
  inputPath: HTMLInputElement;
  browseInputButton: HTMLButtonElement;
  encoderStatus: HTMLParagraphElement;
  convertButton: HTMLButtonElement;
  cancelButton: HTMLButtonElement;
  progress: HTMLProgressElement;
  progressText: HTMLSpanElement;
  status: HTMLPreElement;
}

export function getAppElements(): AppElements {
  return {
    appRoot: requireElement<HTMLElement>("appRoot", HTMLElement),
    inputPath: requireElement<HTMLInputElement>("inputPath", HTMLInputElement),
    browseInputButton: requireElement<HTMLButtonElement>("browseInput", HTMLButtonElement),
    encoderStatus: requireElement<HTMLParagraphElement>("encoderStatus", HTMLParagraphElement),
    convertButton: requireElement<HTMLButtonElement>("convert", HTMLButtonElement),
    cancelButton: requireElement<HTMLButtonElement>("cancel", HTMLButtonElement),
    progress: requireElement<HTMLProgressElement>("progress", HTMLProgressElement),
    progressText: requireElement<HTMLSpanElement>("progressText", HTMLSpanElement),
    status: requireElement<HTMLPreElement>("status", HTMLPreElement)
  };
}
