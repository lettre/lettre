use ::base64::{
    DecodeError,
    engine::{Engine, general_purpose::STANDARD},
};

pub(crate) fn encode<T: AsRef<[u8]>>(input: T) -> String {
    STANDARD.encode(input)
}

pub(crate) fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    STANDARD.decode(input)
}
