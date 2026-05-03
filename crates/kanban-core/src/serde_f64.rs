//! Lossless `f64` round-trip via bit pattern.
//!
//! `serde_json`'s `f64` deserializer is not bit-exact for arbitrary doubles
//! (it can drift by 1 ULP on values like `2.2901265025181274`). This wrapper
//! preserves all 64 bits by encoding the value as its `u64` bit-pattern in
//! lowercase hex.
//!
//! # Use site
//!
//! Annotate fields with `#[serde(with = "crate::serde_f64::bits")]`. This
//! affects only the JSON serialization of the containing struct; the runtime
//! `f64` value is unchanged, and storage that stores the field elsewhere
//! (e.g. `SQLite` `REAL` columns) is also unaffected.

pub(crate) mod bits {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serialize an `f64` as its `u64` bit-pattern in `0x`-prefixed lowercase hex.
    ///
    /// Takes `&f64` rather than `f64` because `serde` requires this signature
    /// for `#[serde(with = "...")]` field attributes.
    ///
    /// # Errors
    ///
    /// Returns the underlying serializer error if encoding fails.
    #[allow(clippy::trivially_copy_pass_by_ref)] // serde_with-style signature is mandatory.
    pub(crate) fn serialize<S: Serializer>(value: &f64, ser: S) -> Result<S::Ok, S::Error> {
        let bits = value.to_bits();
        format!("{bits:#018x}").serialize(ser)
    }

    /// Deserialize an `f64` from its `u64` bit-pattern hex string.
    ///
    /// Accepts either the canonical `0x...` form produced by [`serialize`] or
    /// a raw hex string without the `0x` prefix.
    ///
    /// # Errors
    ///
    /// Returns a deserializer error if the input is not valid hex or is not
    /// representable as a `u64`.
    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<f64, D::Error> {
        let s = String::deserialize(de)?;
        let trimmed = s.strip_prefix("0x").unwrap_or(&s);
        let bits = u64::from_str_radix(trimmed, 16).map_err(serde::de::Error::custom)?;
        Ok(f64::from_bits(bits))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Wrap(#[serde(with = "super::bits")] f64);

    #[test]
    fn round_trips_arbitrary_doubles_bit_exactly() {
        for f in [
            0.0_f64,
            -0.0_f64,
            1.0_f64,
            -1.0_f64,
            std::f64::consts::PI,
            std::f64::consts::E,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::MAX,
            f64::MIN,
            // The 1-ulp drift case the property test was working around.
            2.290_126_502_518_127_4_f64,
        ] {
            let s = serde_json::to_string(&Wrap(f)).unwrap();
            let back: Wrap = serde_json::from_str(&s).unwrap();
            assert_eq!(back.0.to_bits(), f.to_bits(), "round-trip mismatch for {f}");
        }
    }

    #[test]
    fn round_trips_specific_nan_bit_pattern() {
        // NaN != NaN, so we compare bit patterns explicitly. Use a non-canonical
        // payload to prove all 52 mantissa bits survive.
        let nan = f64::from_bits(0x7ff8_0000_dead_beef);
        let s = serde_json::to_string(&Wrap(nan)).unwrap();
        let back: Wrap = serde_json::from_str(&s).unwrap();
        assert_eq!(back.0.to_bits(), nan.to_bits());
    }

    #[test]
    fn serialized_form_is_018_hex() {
        let s = serde_json::to_string(&Wrap(1.0)).unwrap();
        // 1.0 has bit pattern 0x3ff0_0000_0000_0000.
        assert_eq!(s, "\"0x3ff0000000000000\"");
    }

    #[test]
    #[allow(clippy::float_cmp)] // bit-exact round-trip; equality is the contract.
    fn accepts_unprefixed_hex_on_deserialize() {
        let back: Wrap = serde_json::from_str("\"3ff0000000000000\"").unwrap();
        assert_eq!(back.0, 1.0_f64);
    }
}
