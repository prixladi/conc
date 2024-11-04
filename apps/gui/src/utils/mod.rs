pub fn prettify_serializable<'a, T: serde::de::Deserialize<'a> + serde::Serialize>(
    data: &'a T,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&data)
}
