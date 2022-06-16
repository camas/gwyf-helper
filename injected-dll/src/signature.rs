#[allow(dead_code)]
pub type Signature = Vec<Option<u8>>;

#[allow(unused_macros)]
macro_rules! signature {
    ($sig:expr) => {{
        let parts = $sig.split(' ');
        parts
            .map(|part| match part {
                "??" => None,
                part => Some(u8::from_str_radix(part, 16).unwrap()),
            })
            .collect::<Vec<_>>()
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sig_macro() {
        assert_eq!(signature!("0a ?? ff"), vec![Some(0x0a), None, Some(0xff)]);
    }
}
