pub(crate) fn normalize_provider_id(id: &str) -> String {
    id.trim().to_ascii_lowercase().replace('_', "-")
}
