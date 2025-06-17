pub fn prettify_serializable<T: serde::Serialize>(data: T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&data)
}
