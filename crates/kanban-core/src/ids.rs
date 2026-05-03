use uuid::Uuid;

#[must_use]
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

#[must_use]
pub fn format_identifier(prefix: &str, seq: i64) -> String {
    format!("{prefix}-{seq}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_returns_v7_uuids_that_sort_chronologically() {
        let a = new_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = new_id();
        assert_eq!(a.get_version_num(), 7);
        assert_eq!(b.get_version_num(), 7);
        assert!(a < b, "v7 ids should sort by creation time: {a} < {b}");
    }

    #[test]
    fn format_identifier_joins_prefix_and_seq() {
        assert_eq!(format_identifier("KAN", 42), "KAN-42");
        assert_eq!(format_identifier("AUTH", 1), "AUTH-1");
    }
}
