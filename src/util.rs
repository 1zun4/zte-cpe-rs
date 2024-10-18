pub fn bool_to_int<S>(b: &bool, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_i32(if *b { 1 } else { 0 })
}

pub fn bool_to_str<S>(b: &bool, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(if *b { "on" } else { "off" })
}
