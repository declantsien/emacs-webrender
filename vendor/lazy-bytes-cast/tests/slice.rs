use lazy_bytes_cast::{FromByteArray, IntoByteArray, AsByteSlice, ReadByteSlice, FromByteSlice, slice};

#[test]
fn should_to_bytes() {
    let expected = [127u8, 150, 152, 0];
    let var_from: u32 = 9999999;

    assert_eq!(var_from.as_slice(), expected);
    assert_eq!(var_from.into_byte_array(), expected);
}

#[test]
fn should_from_bytes() {
    let expected: u32 = 9999999;
    let bytes = [127u8, 150, 152, 0];

    assert!(u32::from_slice(&bytes[..1]).is_none());
    assert!(u32::read_byte_slice(&mut &bytes[..1]).is_none());

    unsafe {
        assert_eq!(*slice::as_type::<u32>(&bytes[..]).unwrap(), expected);
    }
    assert_eq!(u32::from_slice(&bytes).unwrap(), expected);
    assert_eq!(u32::from_byte_array(bytes), expected);
    assert_eq!(u32::read_byte_slice(&mut &bytes[..]).unwrap(), expected);
}
