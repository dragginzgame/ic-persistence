use super::*;

#[test]
fn byte_codec_borrows_and_preserves_every_encoding() {
    for value in u8::MIN..=u8::MAX {
        let bytes = value.to_bytes_checked().unwrap();
        assert!(matches!(bytes, Cow::Borrowed(_)));
        assert_eq!(bytes.as_ref(), &[value]);
        assert_eq!(u8::from_bytes(bytes).unwrap(), value);
        let owned = value.into_bytes_checked().unwrap();
        assert_eq!(owned, [value]);
        assert_eq!(u8::from_bytes(Cow::Owned(owned)).unwrap(), value);
    }
    for bytes in [&[][..], &[0, 1][..]] {
        assert_eq!(
            u8::from_bytes(Cow::Borrowed(bytes)),
            Err(CodecError::Invalid("integer width"))
        );
    }
}

#[test]
fn boolean_codec_borrows_and_rejects_non_boolean_bytes() {
    for (value, byte) in [(false, 0), (true, 1)] {
        let bytes = value.to_bytes_checked().unwrap();
        assert!(matches!(bytes, Cow::Borrowed(_)));
        assert_eq!(bytes.as_ref(), &[byte]);
        assert_eq!(bool::from_bytes(bytes).unwrap(), value);
        let owned = value.into_bytes_checked().unwrap();
        assert_eq!(owned, [byte]);
        assert_eq!(bool::from_bytes(Cow::Owned(owned)).unwrap(), value);
    }
    for byte in 2..=u8::MAX {
        assert!(bool::from_bytes(Cow::Borrowed(&[byte])).is_err());
    }
    for bytes in [&[][..], &[0, 1][..]] {
        assert!(bool::from_bytes(Cow::Borrowed(bytes)).is_err());
    }
}
