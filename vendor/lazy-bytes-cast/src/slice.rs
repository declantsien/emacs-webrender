//! Slice conversion module
//!
//! Provides utilities to treat arbitrary types as slice of bytes

use core::{slice, mem};

#[inline]
///Access value as slice of bytes
pub unsafe fn as_slice<T: Sized>(val: &T) -> &[u8] {
    slice::from_raw_parts(val as *const _ as *const _, mem::size_of::<T>())
}

#[inline]
///Access value as mutable slice of bytes
pub unsafe fn as_slice_mut<T: Sized>(val: &mut T) -> &mut [u8] {
    slice::from_raw_parts_mut(val as *mut _ as *mut _, mem::size_of::<T>())
}

#[inline]
///Get reference to the value from slice, using `as_type_unchecked`
pub unsafe fn as_type<T: Sized>(slice: &[u8]) -> Option<&T> {
    if mem::size_of::<T>() == slice.len() {
        Some(as_type_unchecked(slice))
    } else {
        None
    }
}

#[inline]
///Get mutable reference to the value from slice, using `as_type_mut_unchecked`
pub unsafe fn as_type_mut<T: Sized>(slice: &mut [u8]) -> Option<&mut T> {
    if mem::size_of::<T>() == slice.len() {
        Some(as_type_mut_unchecked(slice))
    } else {
        None
    }
}

#[inline]
///Get reference to the value from slice
///
///This function is UB if `slice.len() < mem::size_of::<T>()`
///
///Also might be UB if `mem::align_if::<T>() > 1`
pub unsafe fn as_type_unchecked<T: Sized>(slice: &[u8]) -> &T {
    &*(slice.as_ptr() as *const T)
}

#[inline]
///Get mutable reference to the value from slice
///
///This function is UB if `slice.len() < mem::size_of::<T>()`
///
///Also might be UB if `mem::align_if::<T>() > 1`
pub unsafe fn as_type_mut_unchecked<T: Sized>(slice: &mut [u8]) -> &mut T {
    &mut *(slice.as_mut_ptr() as *mut T)
}

/// Trait which should be implemented for types that are safe to treat as byte
///
/// While it is possible to consider all types as bytes, it doesn't make sense for some (e.g.  `Vec`)
pub unsafe trait AsByteSlice: Sized {
    #[inline]
    ///Access value as slice of bytes
    fn as_slice(&self) -> &[u8] {
        unsafe {
            as_slice(self)
        }
    }

    #[inline]
    ///Access value as mutable slice of bytes
    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            as_slice_mut(self)
        }
    }
}

macro_rules! impl_trait {
    ($($type:ty,)+) => {
        $(
            unsafe impl AsByteSlice for $type {}
        )+
    }
}

impl_trait!(u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, usize, isize, i128, u128,);

///Describes a way to read byte slice into particular type.
pub trait ReadByteSlice: crate::FromByteArray + Sized {
    #[inline]
    ///Reads from byte slice, converting consumed bytes into `Self`
    ///
    ///Modifying existing slice by taking away required bytes.
    ///
    ///When slice has insufficient size, returns `None`
    fn read_byte_slice(slice: &mut &[u8]) -> Option<Self> {
        if slice.len() < mem::size_of::<Self::Array>() {
            return None;
        }

        Some(unsafe {
            Self::read_byte_slice_unchecked(slice)
        })
    }

    ///Unsafe version of `read_byte_slice`, that doesn't perform length check.
    unsafe fn read_byte_slice_unchecked(slice: &mut &[u8]) -> Self {
        let (convert, remains) = slice.split_at(mem::size_of::<Self::Array>());
        *slice = remains;
        //This is fine because we know that slice has enough bytes
        let convert = convert.as_ptr() as *const Self::Array;

        Self::from_byte_array(*convert)
    }
}

impl<T: crate::FromByteArray> ReadByteSlice for T {}

///Describes a way to read byte slice into particular type.
pub trait FromByteSlice: crate::FromByteArray + Sized {
    #[inline]
    ///Converts slice into `Self` only if `mem::size_of::<Self>() == slice.len()`
    fn from_slice(slice: &[u8]) -> Option<Self> {
        if mem::size_of::<Self>() == slice.len() {
            Some(unsafe {
                Self::from_slice_unchecked(slice)
            })
        } else {
            None
        }
    }

    #[inline]
    ///Converts slice into `Self` without performing any checks.
    unsafe fn from_slice_unchecked(slice: &[u8]) -> Self {
        let slice = slice.as_ptr() as *const Self::Array;
        Self::from_byte_array(*slice)
    }
}

impl<T: crate::FromByteArray> FromByteSlice for T {}
