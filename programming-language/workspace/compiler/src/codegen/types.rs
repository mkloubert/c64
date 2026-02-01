// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
//
// Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Type conversion utilities for code generation.
//!
//! This module provides functions for converting between different numeric
//! representations, particularly for IEEE-754 binary16 (half-precision float).

/// Convert a decimal string to IEEE-754 binary16 bits.
///
/// Supports formats like "3.14", "0.5", "1.5e3", "2.0e-5".
pub fn decimal_string_to_binary16(s: &str) -> u16 {
    // Parse the string as f64, then convert to binary16
    let value: f64 = s.parse().unwrap_or(0.0);
    f64_to_binary16(value)
}

/// Convert an f64 value to IEEE-754 binary16 bits.
///
/// IEEE-754 binary16 format:
/// - Sign: 1 bit (bit 15)
/// - Exponent: 5 bits (bits 14-10), bias = 15
/// - Mantissa: 10 bits (bits 9-0), implicit leading 1 for normalized
pub fn f64_to_binary16(value: f64) -> u16 {
    // Handle special cases
    if value.is_nan() {
        return 0x7E00; // Canonical NaN
    }
    if value.is_infinite() {
        return if value > 0.0 { 0x7C00 } else { 0xFC00 };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            0x8000
        } else {
            0x0000
        };
    }

    let sign = if value < 0.0 { 1u16 } else { 0u16 };
    let abs_value = value.abs();

    // Check for overflow to infinity
    if abs_value > 65504.0 {
        return (sign << 15) | 0x7C00;
    }

    // Check for underflow to zero (smallest subnormal is ~5.96e-8)
    if abs_value < 5.96e-8 {
        return sign << 15;
    }

    // Calculate exponent and mantissa
    let bits = abs_value.to_bits();
    let f64_exp = ((bits >> 52) & 0x7FF) as i32;
    let f64_mant = bits & 0xFFFFFFFFFFFFF;

    // Convert f64 exponent (bias 1023) to binary16 exponent (bias 15)
    let exp = f64_exp - 1023 + 15;

    if exp <= 0 {
        // Subnormal number
        let shift = 1 - exp;
        if shift > 10 {
            return sign << 15; // Too small, becomes zero
        }
        // Subnormal: mantissa = (1.mant >> shift), no implicit 1
        let mant = ((0x400 | (f64_mant >> 42)) >> shift) & 0x3FF;
        return (sign << 15) | (mant as u16);
    }

    if exp >= 31 {
        // Overflow to infinity
        return (sign << 15) | 0x7C00;
    }

    // Normal number: take top 10 bits of f64 mantissa
    let mant = (f64_mant >> 42) & 0x3FF;

    // Round to nearest even
    let round_bit = (f64_mant >> 41) & 1;
    let sticky_bits = f64_mant & 0x1FFFFFFFFFF;
    let mant = if round_bit == 1 && (sticky_bits != 0 || (mant & 1) == 1) {
        mant + 1
    } else {
        mant
    };

    // Check if rounding caused overflow
    if mant > 0x3FF {
        let exp = exp + 1;
        if exp >= 31 {
            return (sign << 15) | 0x7C00; // Overflow to infinity
        }
        return (sign << 15) | ((exp as u16) << 10);
    }

    (sign << 15) | ((exp as u16) << 10) | (mant as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_to_binary16_zero() {
        assert_eq!(f64_to_binary16(0.0), 0x0000);
    }

    #[test]
    fn test_f64_to_binary16_one() {
        assert_eq!(f64_to_binary16(1.0), 0x3C00);
    }

    #[test]
    fn test_f64_to_binary16_negative() {
        // -1.0 should have sign bit set
        let result = f64_to_binary16(-1.0);
        assert_eq!(result & 0x8000, 0x8000);
    }

    #[test]
    fn test_f64_to_binary16_infinity() {
        assert_eq!(f64_to_binary16(f64::INFINITY), 0x7C00);
        assert_eq!(f64_to_binary16(f64::NEG_INFINITY), 0xFC00);
    }

    #[test]
    fn test_f64_to_binary16_nan() {
        let result = f64_to_binary16(f64::NAN);
        assert_eq!(result, 0x7E00);
    }

    #[test]
    fn test_decimal_string_to_binary16() {
        // Test basic parsing
        let result = decimal_string_to_binary16("1.0");
        assert_eq!(result, f64_to_binary16(1.0));
    }
}
