use serde::de::Error;
use serde::{Deserializer, Serializer};
use std::fmt;
use std::fmt::Formatter;

#[derive(Copy, Clone,Eq, PartialEq)]
pub enum AsNumber {
    Small(u16),
    Extended(u32),
}

/// Transition AS number to be used as peer AS when peers determine AS4 number capability
pub const AS_TRANS: AsNumber = AsNumber::Small(23456);

impl Default for AsNumber {
    fn default() -> Self {
        AS_TRANS
    }
}

impl From<u16> for AsNumber {
    fn from(value: u16) -> Self {
        AsNumber::Small(value)
    }
}

impl From<u32> for AsNumber {
    fn from(value: u32) -> Self {
        if value <= 65535 { AsNumber::Small(value as u16) } else { AsNumber::Extended(value) }
    }
}

impl TryFrom<u64> for AsNumber {
    type Error = crate::shared::error::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value <= 65535 {
            Ok(AsNumber::Small(value as u16))
        } else if value <= std::u32::MAX as u64 {
            Ok(AsNumber::Extended(value as u32))
        } else {
            Err(crate::shared::error::Error::InvalidAsNumberError { as_number : value as i64})
        }
    }
}

impl fmt::Display for AsNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AsNumber::Small(number) => number.fmt(f),
            AsNumber::Extended(number) => number.fmt(f)
        }
    }
}

impl fmt::Debug for AsNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl serde::Serialize for AsNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match *self {
            AsNumber::Small(number) => serializer.serialize_u16(number),
            AsNumber::Extended(number) => serializer.serialize_u32(number),
        }
    }
}

impl<'de> serde::Deserialize<'de> for AsNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        deserializer.deserialize_any(AsNumberVisitor)
    }
}

struct AsNumberVisitor;

impl<'de> serde::de::Visitor<'de> for AsNumberVisitor {
    type Value = AsNumber;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a positive integer between 1 and 2^^31")
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v > 0 {
            Ok(AsNumber::from(v as u16))
        } else {
            Err(E::custom(format!("AS Number out of range: {}", v)))
        }
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v > 0 {
            Ok(AsNumber::from(v as u16))
        } else {
            Err(E::custom(format!("AS Number out of range: {}", v)))
        }
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v > 0 {
            Ok(AsNumber::from(v as u32))
        } else {
            Err(E::custom(format!("AS Number out of range: {}", v)))
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v > 0 {
            Ok(AsNumber::from(v as u32))
        } else {
            Err(E::custom(format!("AS Number out of range: {}", v)))
        }
    }


    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(AsNumber::from(v as u16))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(AsNumber::from(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(AsNumber::from(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match AsNumber::try_from(v) {
            Ok(v) => Ok(v),
            Err(_) => Err(E::custom(format!("AS Number out of range: {}", v))),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::Result;

    #[test]
    fn should_yield_small_as_number() {
        match AsNumber::from(4096u16) {
            AsNumber::Small(port_number) => assert_eq!(port_number, 4096),
            AsNumber::Extended(_) => panic!("Should yield small AS number")
        }
    }

    #[test]
    fn should_yield_large_as_number_from_u32_number() {
        match AsNumber::from(131072u32) {
            AsNumber::Small(_) => panic!("Should yield small AS number"),
            AsNumber::Extended(port_number) => assert_eq!(port_number, 131072)
        }
    }

    #[test]
    fn should_yield_small_as_number_from_u16_number() {
        match AsNumber::from(4096u32) {
            AsNumber::Small(port_number) => assert_eq!(port_number, 4096),
            AsNumber::Extended(_) => panic!("Should yield small AS number")
        }
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestData {
        as_number : AsNumber,
    }

    #[test]
    fn should_deserialize_tiny_as_number() {
        let data = r#"
        {
            "asNumber": 1
        }"#;


        let td: TestData = serde_json::from_str(data).unwrap();

        match td.as_number {
            AsNumber::Small(port_number) => assert_eq!(port_number, 1),
            AsNumber::Extended(_) => panic!("Should yield small AS number")
        }
    }

    #[test]
    fn should_deserialize_small_as_number() {
        let data = r#"
        {
            "asNumber": 4096
        }"#;


        let td: TestData = serde_json::from_str(data).unwrap();

        match td.as_number {
            AsNumber::Small(port_number) => assert_eq!(port_number, 4096),
            AsNumber::Extended(_) => panic!("Should yield small AS number")
        }
    }

    #[test]
    fn should_deserialize_large_as_number() {
        let data = r#"
        {
            "asNumber": 131072
        }"#;


        let td: TestData = serde_json::from_str(data).unwrap();

        match td.as_number {
            AsNumber::Small(_) => panic!("Should yield small AS number"),
            AsNumber::Extended(port_number) => assert_eq!(port_number, 131072)
        }
    }

    #[test]
    fn fail_deserialize_negative_tiny_as_number() {
        let data = r#"
        {
            "asNumber": -1
        }"#;

        let td : Result<TestData> = serde_json::from_str(data);

        match td {
            Ok(_) => panic!("Should fail"),
            Err(_) => ()
        }
    }

    #[test]
    fn fail_deserialize_negative_small_as_number() {
        let data = r#"
        {
            "asNumber": -4096
        }"#;

        let td : Result<TestData> = serde_json::from_str(data);

        match td {
            Ok(_) => panic!("Should fail"),
            Err(_) => ()
        }
    }

    #[test]
    fn fail_deserialize_negative_large_as_number() {
        let data = r#"
        {
            "asNumber": -131072
        }"#;

        let td : Result<TestData> = serde_json::from_str(data);

        match td {
            Ok(_) => panic!("Should fail"),
            Err(_) => ()
        }
    }
}