pub const ERROR_PREFIX: &str = "VID2AV1_ERROR";

pub const ERROR_CODE_CANCELED_BY_USER: &str = "CANCELED_BY_USER";
pub const ERROR_CODE_NO_ACTIVE_CONVERSION: &str = "NO_ACTIVE_CONVERSION";

const CANCELED_BY_USER_MESSAGE: &str = "Conversion canceled by user.";
const NO_ACTIVE_CONVERSION_MESSAGE: &str = "No conversion is currently running.";

pub fn encode_error(code: &str, message: &str) -> String {
    format!("{ERROR_PREFIX}|{code}|{message}")
}

pub fn error_canceled_by_user() -> String {
    encode_error(ERROR_CODE_CANCELED_BY_USER, CANCELED_BY_USER_MESSAGE)
}

pub fn error_no_active_conversion() -> String {
    encode_error(
        ERROR_CODE_NO_ACTIVE_CONVERSION,
        NO_ACTIVE_CONVERSION_MESSAGE,
    )
}

pub fn is_error_code(error: &str, code: &str) -> bool {
    let mut parts = error.splitn(3, '|');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(prefix), Some(found_code), Some(_)) if prefix == ERROR_PREFIX && found_code == code
    )
}
