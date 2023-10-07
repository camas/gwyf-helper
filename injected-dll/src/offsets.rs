pub(crate) const DEFAULT_OFFSET: isize = 0x180000000;

include!(concat!(env!("OUT_DIR"), "/offsets-const.rs"));

macro_rules! method_offset {
    ($name:ident) => {{
        crate::offsets::methods::$name - crate::offsets::DEFAULT_OFFSET
    }};
}

macro_rules! api_offset {
    ($name:ident) => {{
        // 0xc00 because some il2cpp shit I don't understand
        crate::offsets::apis::$name - crate::offsets::DEFAULT_OFFSET - 0xc00
    }};
}

pub(crate) use api_offset;
pub(crate) use method_offset;
