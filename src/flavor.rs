use jomini::{BinaryFlavor, Encoding, Utf8Encoding};

pub struct Hoi4Flavor;

impl Encoding for Hoi4Flavor {
    fn decode<'a>(&self, data: &'a [u8]) -> std::borrow::Cow<'a, str> {
        Utf8Encoding::decode(data)
    }
}

impl BinaryFlavor for Hoi4Flavor {
    fn visit_f32(&self, data: [u8; 4]) -> f32 {
        i32::from_le_bytes(data) as f32 / 1000.0
    }

    fn visit_f64(&self, data: [u8; 8]) -> f64 {
        let val = i64::from_le_bytes(data) as f64 / 32768.0;
        (val * 10_0000.0).floor() / 10_0000.0
    }
}
