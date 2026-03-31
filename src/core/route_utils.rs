pub fn encode_route_segment(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

pub fn decode_route_segment(value: &str) -> String {
    urlencoding::decode(value)
        .map(|decoded| decoded.into_owned())
        .unwrap_or_else(|_| value.to_string())
}
