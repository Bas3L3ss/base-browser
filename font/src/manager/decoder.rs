pub struct FontDecoder;

impl FontDecoder {
    pub fn decode_woff(bytes: &[u8]) -> Option<Vec<u8>> {
        match woff::convert_woff_to_ttf(&bytes) {
            Ok(ttf_bytes) => Some(ttf_bytes),
            Err(_) => None,
        }
    }

    pub fn decode_woff2(bytes: &mut [u8]) -> Option<Vec<u8>> {
        match woff2::convert_woff2_to_ttf(bytes) {
            Ok(ttf_bytes) => Some(ttf_bytes),
            Err(_) => None,
        }
    }
}
