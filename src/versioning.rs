pub const LANGUAGE_DEFAULT: &str = "1.0";
pub const LANGUAGE_SUPPORT_RANGE: &str = ">=1.0 <2.0";

pub fn language_line() -> String {
    format!(
        "SCULPT language {} (supports {})",
        LANGUAGE_DEFAULT, LANGUAGE_SUPPORT_RANGE
    )
}
