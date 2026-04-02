const ROUTE_SLASH_PLACEHOLDER: &str = "__doxle_route_slash__";
pub fn encode_route_segment(value: &str) -> String {
    let safe = value.replace("/", ROUTE_SLASH_PLACEHOLDER);
    urlencoding::encode(&safe).into_owned()
}

pub fn decode_route_segment(value: &str) -> String {
    urlencoding::decode(value)
        .map(|decoded| decoded.into_owned())
        .unwrap_or_else(|_| value.to_string())
        .replace(ROUTE_SLASH_PLACEHOLDER, "/")
}
