pub trait ParseRadix: std::str::FromStr {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

macro_rules! impl_parse_radix_signed {
    ($signed:ty, $unsigned:ty, $limit:expr, $upscale:ty, $wrap_around:expr) => {
        impl ParseRadix for $signed {
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                let val = <$unsigned>::from_str_radix(s, radix)?;
                if val <= $limit {
                    Ok(val as $signed)
                } else {
                    Ok((val as $upscale - $wrap_around) as $signed)
                }
            }
        }
    };
}

impl_parse_radix_signed!(i8, u8, 0x7F, i16, 0x100);
impl_parse_radix_signed!(i16, u16, 0x7FFF, i32, 0x10000);
impl_parse_radix_signed!(i32, u32, 0x7FFFFFFF, i64, 0x100000000);
impl_parse_radix_signed!(
    i64,
    u64,
    0x7FFFFFFFFFFFFFFF,
    i128,
    0x10000000000000000
);

macro_rules! impl_parse_radix_for {
    ($t:ty) => {
        impl ParseRadix for $t {
            fn from_str_radix(
                s: &str,
                radix: u32,
            ) -> Result<Self, <Self as std::str::FromStr>::Err> {
                <$t>::from_str_radix(s, radix)
            }
        }
    };
}

impl_parse_radix_for!(u8);
impl_parse_radix_for!(u16);
impl_parse_radix_for!(u32);
impl_parse_radix_for!(u64);

pub fn parse_number<T: ParseRadix + Default>(s: &str) -> T {
    let maybe_number = if s.starts_with("0x") || s.starts_with("0X") {
        T::from_str_radix(&s[2..], 16)
    } else {
        s.parse()
    };

    maybe_number
        .inspect_err(|_cause| {
            log::warn!("Failed to parse number. Rollback to default value",)
        })
        .unwrap_or_default()
}
