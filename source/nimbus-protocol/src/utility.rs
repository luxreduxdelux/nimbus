use serde::{Deserialize, Serialize};

pub fn serialize<T: Serialize>(value: &T) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    ciborium::into_writer(value, &mut data)?;
    Ok(data)
}

pub fn deserialize<T: for<'a> Deserialize<'a>>(value: &[u8]) -> anyhow::Result<T> {
    Ok(ciborium::from_reader(value)?)
}
