const CANCELED_ERROR_TEXT = "conversion canceled by user";
const NO_ACTIVE_CONVERSION_TEXT = "no conversion is currently running";

export function isCancellationErrorMessage(message: string): boolean {
  return message.toLowerCase().includes(CANCELED_ERROR_TEXT);
}

export function isNoActiveConversionErrorMessage(message: string): boolean {
  return message.toLowerCase().includes(NO_ACTIVE_CONVERSION_TEXT);
}
