pub mod cfb;

macro_rules! impl_for_hex_debug {
    ($type:ident, $hex_mask:literal) => {
        impl fmt::Debug for $type {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&format!(concat!(stringify!($type), "(0x{:0", $hex_mask, "X})") , self.0))
            }
        }
    };
}

pub(crate) use impl_for_hex_debug;

/// Adds entries to a debug map formatter, with the getter method names being the keys.
///
/// The first argument is the debug map formatter, the second argument is the receiver of the getter
/// methods, and the rest of the arguments are getter method identifiers.
///
/// # Examples
///
/// ```
/// impl fmt::Debug for Cfb {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         let mut fmt = f.debug_map();
///         crate::debug_map_method_reflection!(fmt, self, sector_size, header);
///         fmt.finish()
///     }
/// }
/// ```
macro_rules! debug_map_method_reflection {
    ($fmt:tt, $receiver:tt, $first:ident, $($methods:tt)*) => {
        $fmt.entry(&stringify!($first), &$receiver.$first());
        crate::debug_map_method_reflection!($fmt, $receiver, $($methods)*);
    };

    ($fmt:tt, $receiver:tt, $method:ident) => {
        $fmt.entry(&stringify!($method), &$receiver.$method());
    }
}

pub(crate) use debug_map_method_reflection;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn read_header() {
        let cfb = cfb::Cfb::from_path("/Users/resonancel/Desktop/testing.doc");
        println!("{:?}", cfb);
    }
}
