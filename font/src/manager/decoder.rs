pub struct FontDecoder;

impl FontDecoder {
    pub fn decode_woff(bytes: &[u8]) -> Option<Vec<u8>> {
        woff::version1::decompress(&bytes)
    }

    pub fn decode_woff2(bytes: &[u8]) -> Option<Vec<u8>> {
        woff::version2::decompress(&bytes)
    }
}
