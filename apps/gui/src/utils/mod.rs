pub fn prettify_json<'a, T: serde::de::Deserialize<'a> + serde::Serialize>(
    data: &'a str,
) -> Result<String, serde_json::Error> {
    serde_json::from_str::<T>(data).and_then(|d| serde_json::to_string_pretty(&d))
}
