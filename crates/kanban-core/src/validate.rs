use crate::error::{Error, Result, ValidationError};

/// Project prefix: 3–6 uppercase ASCII letters, e.g., "KAN", "AUTH".
///
/// # Errors
/// Returns [`Error::Validation`] if the value is empty, not 3–6 chars, or contains
/// non-uppercase-ASCII characters.
pub fn project_prefix(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(invalid("prefix", "must not be empty"));
    }
    if !(3..=6).contains(&value.len()) {
        return Err(invalid("prefix", "must be 3–6 characters"));
    }
    if !value.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(invalid("prefix", "must be uppercase ASCII letters only"));
    }
    Ok(())
}

/// Non-empty trimmed string field. Returns the trimmed value on success.
///
/// # Errors
/// Returns [`Error::Validation`] if the trimmed value is empty.
pub fn nonempty_field<'a>(field: &str, value: &'a str) -> Result<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    Ok(trimmed)
}

/// Hex color: `#RRGGBB`.
///
/// # Errors
/// Returns [`Error::Validation`] if the value is not a 7-character `#RRGGBB` hex string.
pub fn hex_color(value: &str) -> Result<()> {
    if value.len() != 7 || !value.starts_with('#') {
        return Err(invalid(
            "color",
            "must be a 7-char hex string starting with #",
        ));
    }
    if !value[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid("color", "must contain only hex digits"));
    }
    Ok(())
}

fn invalid(field: &str, reason: &str) -> Error {
    Error::Validation(ValidationError {
        field: field.to_string(),
        reason: reason.to_string(),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn project_prefix_accepts_valid() {
        assert!(project_prefix("KAN").is_ok());
        assert!(project_prefix("AUTH").is_ok());
        assert!(project_prefix("ABCDEF").is_ok());
    }

    #[test]
    fn project_prefix_rejects_invalid() {
        assert!(project_prefix("").is_err());
        assert!(project_prefix("ab").is_err());
        assert!(project_prefix("kan").is_err());
        assert!(project_prefix("KAN1").is_err());
        assert!(project_prefix("TOOOLONG").is_err());
    }

    #[test]
    fn nonempty_field_trims() {
        assert_eq!(nonempty_field("name", "  hi  ").unwrap(), "hi");
    }

    #[test]
    fn nonempty_field_rejects_empty_or_whitespace() {
        assert!(nonempty_field("name", "").is_err());
        assert!(nonempty_field("name", "   ").is_err());
    }

    #[test]
    fn hex_color_accepts_valid() {
        assert!(hex_color("#000000").is_ok());
        assert!(hex_color("#FFFFFF").is_ok());
        assert!(hex_color("#3b82f6").is_ok());
    }

    #[test]
    fn hex_color_rejects_invalid() {
        assert!(hex_color("000000").is_err());
        assert!(hex_color("#00").is_err());
        assert!(hex_color("#GGGGGG").is_err());
    }
}
