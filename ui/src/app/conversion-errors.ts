const APP_ERROR_PREFIX = "VID2AV1_ERROR";
const CANCELED_BY_USER_CODE = "CANCELED_BY_USER";
const NO_ACTIVE_CONVERSION_CODE = "NO_ACTIVE_CONVERSION";
const CANCELED_ERROR_TEXT = "conversion canceled by user";
const NO_ACTIVE_CONVERSION_TEXT = "no conversion is currently running";

interface ParsedAppError {
  code: string;
  message: string;
}

export function parseAppErrorMessage(message: string): ParsedAppError | null {
  const expectedPrefix = `${APP_ERROR_PREFIX}|`;
  if (!message.startsWith(expectedPrefix)) {
    return null;
  }

  const remaining = message.slice(expectedPrefix.length);
  const separatorIndex = remaining.indexOf("|");
  if (separatorIndex <= 0) {
    return null;
  }

  const code = remaining.slice(0, separatorIndex).trim();
  const parsedMessage = remaining.slice(separatorIndex + 1).trim();
  if (code.length === 0 || parsedMessage.length === 0) {
    return null;
  }

  return { code, message: parsedMessage };
}

export function toDisplayErrorMessage(message: string): string {
  return parseAppErrorMessage(message)?.message ?? message;
}

export function isCancellationErrorMessage(message: string): boolean {
  const parsed = parseAppErrorMessage(message);
  if (parsed) {
    return parsed.code === CANCELED_BY_USER_CODE;
  }

  return message.toLowerCase().includes(CANCELED_ERROR_TEXT);
}

export function isNoActiveConversionErrorMessage(message: string): boolean {
  const parsed = parseAppErrorMessage(message);
  if (parsed) {
    return parsed.code === NO_ACTIVE_CONVERSION_CODE;
  }

  return message.toLowerCase().includes(NO_ACTIVE_CONVERSION_TEXT);
}
