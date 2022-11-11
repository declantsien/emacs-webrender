//! Array conversion

use core::mem;

///Describes conversion to byte array.
pub trait IntoByteArray: Copy {
    ///Type into which to convert.
    type Array: Copy + AsRef<[u8]> + AsMut<[u8]> + core::borrow::BorrowMut<[u8]> + core::fmt::Debug;

    ///Performs conversion of self into `Array`.
    fn into_byte_array(self) -> Self::Array;
}

///Describes conversion from byte array.
pub unsafe trait FromByteArray {
    ///Type into which to convert.
    type Array: Copy + AsRef<[u8]> + AsMut<[u8]> + core::borrow::BorrowMut<[u8]> + core::fmt::Debug;

    ///Converts array to self.
    fn from_byte_array(arr: Self::Array) -> Self;
}

macro_rules! impl_trait {
    ($($type:ty,)+) => {
        $(
            impl IntoByteArray for $type {
                type Array = [u8; mem::size_of::<$type>()];

                #[inline(always)]
                fn into_byte_array(self) -> Self::Array {
                    unsafe { mem::transmute(self) }
                }
            }

            unsafe impl FromByteArray for $type {
                type Array = [u8; mem::size_of::<$type>()];

                #[inline(always)]
                fn from_byte_array(arr: Self::Array) -> Self {
                    unsafe { mem::transmute(arr) }
                }
            }
        )+
    }
}

impl_trait!(u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, usize, isize, u128, i128,);
