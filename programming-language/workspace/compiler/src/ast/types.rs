// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
// Copyright (C) 2026  Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Type definitions for the Cobra64 compiler.

/// A type in the Cobra64 language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 8-bit unsigned integer (0-255).
    Byte,
    /// 16-bit unsigned integer (0-65535).
    Word,
    /// 8-bit signed integer (-128 to 127).
    Sbyte,
    /// 16-bit signed integer (-32768 to 32767).
    Sword,
    /// 16-bit fixed-point (12.4 format, range -2048.0 to +2047.9375).
    Fixed,
    /// 16-bit IEEE-754 binary16 floating-point (range ±65504).
    Float,
    /// Boolean value.
    Bool,
    /// Text string.
    String,
    /// Byte array (unsigned 8-bit elements).
    ByteArray(Option<u16>),
    /// Word array (unsigned 16-bit elements).
    WordArray(Option<u16>),
    /// Bool array (boolean elements).
    BoolArray(Option<u16>),
    /// Sbyte array (signed 8-bit elements).
    SbyteArray(Option<u16>),
    /// Sword array (signed 16-bit elements).
    SwordArray(Option<u16>),
    /// Void (no value, for functions).
    Void,
}

impl Type {
    /// Get the size of this type in bytes.
    pub fn size(&self) -> usize {
        match self {
            Type::Byte | Type::Sbyte | Type::Bool => 1,
            Type::Word | Type::Sword | Type::Fixed | Type::Float => 2,
            Type::String => 2, // String variable stores a 16-bit pointer
            Type::ByteArray(Some(n)) => *n as usize,
            Type::ByteArray(None) => 0, // Unknown size
            Type::WordArray(Some(n)) => (*n as usize) * 2,
            Type::WordArray(None) => 0,               // Unknown size
            Type::BoolArray(Some(n)) => *n as usize,  // 1 byte per bool
            Type::BoolArray(None) => 0,               // Unknown size
            Type::SbyteArray(Some(n)) => *n as usize, // 1 byte per sbyte
            Type::SbyteArray(None) => 0,              // Unknown size
            Type::SwordArray(Some(n)) => (*n as usize) * 2, // 2 bytes per sword
            Type::SwordArray(None) => 0,              // Unknown size
            Type::Void => 0,
        }
    }

    /// Check if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Byte | Type::Word | Type::Sbyte | Type::Sword)
    }

    /// Check if this is a fixed-point type.
    pub fn is_fixed(&self) -> bool {
        matches!(self, Type::Fixed)
    }

    /// Check if this is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float)
    }

    /// Check if this is a numeric type (integer, fixed, or float).
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_fixed() || self.is_float()
    }

    /// Check if this is a signed type.
    pub fn is_signed(&self) -> bool {
        matches!(self, Type::Sbyte | Type::Sword | Type::Fixed | Type::Float)
    }

    /// Check if this is an 8-bit type.
    pub fn is_8bit(&self) -> bool {
        matches!(self, Type::Byte | Type::Sbyte | Type::Bool)
    }

    /// Check if this is a 16-bit type.
    pub fn is_16bit(&self) -> bool {
        matches!(
            self,
            Type::Word | Type::Sword | Type::Fixed | Type::Float | Type::String
        )
    }

    /// Check if this is an array type.
    pub fn is_array(&self) -> bool {
        matches!(
            self,
            Type::ByteArray(_)
                | Type::WordArray(_)
                | Type::BoolArray(_)
                | Type::SbyteArray(_)
                | Type::SwordArray(_)
        )
    }

    /// Get the element type if this is an array.
    pub fn element_type(&self) -> Option<Type> {
        match self {
            Type::ByteArray(_) => Some(Type::Byte),
            Type::WordArray(_) => Some(Type::Word),
            Type::BoolArray(_) => Some(Type::Bool),
            Type::SbyteArray(_) => Some(Type::Sbyte),
            Type::SwordArray(_) => Some(Type::Sword),
            _ => None,
        }
    }

    /// Check if a value of this type can be assigned to another type.
    pub fn is_assignable_to(&self, target: &Type) -> bool {
        if self == target {
            return true;
        }

        // Integer type promotion rules
        match (self, target) {
            // Unsigned to larger unsigned
            (Type::Byte, Type::Word) => true,
            // Unsigned to larger signed (safe: byte 0-255 fits in sword -32768..32767)
            (Type::Byte, Type::Sword) => true,
            // Signed to larger signed
            (Type::Sbyte, Type::Sword) => true,
            // Byte to sbyte: allowed for literals, range checked at compile time
            // This allows `x: sbyte = 127` to work (127 is a valid sbyte value)
            (Type::Byte, Type::Sbyte) => true,
            // Word to sword: allowed for literals, range checked at compile time
            // This allows `x: sword = 32767` to work (32767 is a valid sword value)
            (Type::Word, Type::Sword) => true,

            // Integer to fixed: allowed (value becomes N.0)
            // Range is checked at compile time (-2048 to 2047 for integer part)
            (Type::Byte, Type::Fixed) => true,
            (Type::Sbyte, Type::Fixed) => true,
            (Type::Word, Type::Fixed) => true, // Range checked at compile time
            (Type::Sword, Type::Fixed) => true, // Range checked at compile time

            // Integer to float: allowed (may lose precision for large values)
            (Type::Byte, Type::Float) => true,
            (Type::Sbyte, Type::Float) => true,
            (Type::Word, Type::Float) => true,
            (Type::Sword, Type::Float) => true,

            // Fixed to float: allowed (fixed range fits in float range)
            (Type::Fixed, Type::Float) => true,

            // Array type compatibility:
            // - byte[n] can be assigned to byte[] (unsized accepts any size)
            // - byte[n] can be assigned to byte[n] (same size) - handled by == check above
            // - byte[] cannot be assigned to byte[n] (unsized to sized is not allowed)
            (Type::ByteArray(Some(_)), Type::ByteArray(None)) => true,
            (Type::WordArray(Some(_)), Type::WordArray(None)) => true,
            (Type::BoolArray(Some(_)), Type::BoolArray(None)) => true,
            (Type::SbyteArray(Some(_)), Type::SbyteArray(None)) => true,
            (Type::SwordArray(Some(_)), Type::SwordArray(None)) => true,

            // Float to fixed: requires explicit cast (potential precision/range loss)
            // Fixed to integer: requires explicit cast (truncation)
            // Float to integer: requires explicit cast (truncation)
            _ => false,
        }
    }

    /// Get the result type of a binary operation between two types.
    pub fn binary_result_type(left: &Type, right: &Type) -> Option<Type> {
        match (left, right) {
            // Same types
            (Type::Byte, Type::Byte) => Some(Type::Byte),
            (Type::Word, Type::Word) => Some(Type::Word),
            (Type::Sbyte, Type::Sbyte) => Some(Type::Sbyte),
            (Type::Sword, Type::Sword) => Some(Type::Sword),
            (Type::Fixed, Type::Fixed) => Some(Type::Fixed),
            (Type::Float, Type::Float) => Some(Type::Float),
            (Type::Bool, Type::Bool) => Some(Type::Bool),

            // Mixed unsigned: promote to larger
            (Type::Byte, Type::Word) | (Type::Word, Type::Byte) => Some(Type::Word),

            // Mixed signed: promote to larger
            (Type::Sbyte, Type::Sword) | (Type::Sword, Type::Sbyte) => Some(Type::Sword),

            // Mixed signed/unsigned of same size: promote to signed
            // byte + sbyte -> sbyte (comparison/arithmetic with literals like 0)
            (Type::Byte, Type::Sbyte) | (Type::Sbyte, Type::Byte) => Some(Type::Sbyte),
            // word + sword -> sword
            (Type::Word, Type::Sword) | (Type::Sword, Type::Word) => Some(Type::Sword),

            // Mixed sizes, different signedness: promote to larger signed
            // byte + sword -> sword
            (Type::Byte, Type::Sword) | (Type::Sword, Type::Byte) => Some(Type::Sword),
            // sbyte + word -> sword
            (Type::Sbyte, Type::Word) | (Type::Word, Type::Sbyte) => Some(Type::Sword),

            // Fixed with integers: promote to fixed
            (Type::Fixed, Type::Byte)
            | (Type::Byte, Type::Fixed)
            | (Type::Fixed, Type::Sbyte)
            | (Type::Sbyte, Type::Fixed)
            | (Type::Fixed, Type::Word)
            | (Type::Word, Type::Fixed)
            | (Type::Fixed, Type::Sword)
            | (Type::Sword, Type::Fixed) => Some(Type::Fixed),

            // Float with integers: promote to float
            (Type::Float, Type::Byte)
            | (Type::Byte, Type::Float)
            | (Type::Float, Type::Sbyte)
            | (Type::Sbyte, Type::Float)
            | (Type::Float, Type::Word)
            | (Type::Word, Type::Float)
            | (Type::Float, Type::Sword)
            | (Type::Sword, Type::Float) => Some(Type::Float),

            // Fixed + Float: promote to float (float has larger range)
            (Type::Fixed, Type::Float) | (Type::Float, Type::Fixed) => Some(Type::Float),

            // Other combinations not allowed
            _ => None,
        }
    }

    /// Get a human-readable name for this type.
    pub fn name(&self) -> &'static str {
        match self {
            Type::Byte => "byte",
            Type::Word => "word",
            Type::Sbyte => "sbyte",
            Type::Sword => "sword",
            Type::Fixed => "fixed",
            Type::Float => "float",
            Type::Bool => "bool",
            Type::String => "string",
            Type::ByteArray(_) => "byte[]",
            Type::WordArray(_) => "word[]",
            Type::BoolArray(_) => "bool[]",
            Type::SbyteArray(_) => "sbyte[]",
            Type::SwordArray(_) => "sword[]",
            Type::Void => "void",
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::ByteArray(Some(n)) => write!(f, "byte[{}]", n),
            Type::ByteArray(None) => write!(f, "byte[]"),
            Type::WordArray(Some(n)) => write!(f, "word[{}]", n),
            Type::WordArray(None) => write!(f, "word[]"),
            Type::BoolArray(Some(n)) => write!(f, "bool[{}]", n),
            Type::BoolArray(None) => write!(f, "bool[]"),
            Type::SbyteArray(Some(n)) => write!(f, "sbyte[{}]", n),
            Type::SbyteArray(None) => write!(f, "sbyte[]"),
            Type::SwordArray(Some(n)) => write!(f, "sword[{}]", n),
            Type::SwordArray(None) => write!(f, "sword[]"),
            _ => write!(f, "{}", self.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_size() {
        assert_eq!(Type::Byte.size(), 1);
        assert_eq!(Type::Word.size(), 2);
        assert_eq!(Type::Bool.size(), 1);
        assert_eq!(Type::Sbyte.size(), 1);
        assert_eq!(Type::Sword.size(), 2);
        assert_eq!(Type::Fixed.size(), 2);
        assert_eq!(Type::Float.size(), 2);
        assert_eq!(Type::String.size(), 2); // Pointer size
        assert_eq!(Type::Void.size(), 0);
    }

    #[test]
    fn test_array_size() {
        assert_eq!(Type::ByteArray(Some(10)).size(), 10);
        assert_eq!(Type::ByteArray(None).size(), 0);
        assert_eq!(Type::WordArray(Some(10)).size(), 20);
        assert_eq!(Type::WordArray(None).size(), 0);
        assert_eq!(Type::BoolArray(Some(10)).size(), 10);
        assert_eq!(Type::BoolArray(None).size(), 0);
        assert_eq!(Type::SbyteArray(Some(10)).size(), 10);
        assert_eq!(Type::SbyteArray(None).size(), 0);
        assert_eq!(Type::SwordArray(Some(10)).size(), 20);
        assert_eq!(Type::SwordArray(None).size(), 0);
    }

    #[test]
    fn test_is_integer() {
        assert!(Type::Byte.is_integer());
        assert!(Type::Word.is_integer());
        assert!(Type::Sbyte.is_integer());
        assert!(Type::Sword.is_integer());
        assert!(!Type::Fixed.is_integer());
        assert!(!Type::Float.is_integer());
        assert!(!Type::Bool.is_integer());
        assert!(!Type::String.is_integer());
        assert!(!Type::Void.is_integer());
    }

    #[test]
    fn test_is_fixed() {
        assert!(Type::Fixed.is_fixed());
        assert!(!Type::Float.is_fixed());
        assert!(!Type::Byte.is_fixed());
        assert!(!Type::Word.is_fixed());
    }

    #[test]
    fn test_is_float() {
        assert!(Type::Float.is_float());
        assert!(!Type::Fixed.is_float());
        assert!(!Type::Byte.is_float());
        assert!(!Type::Word.is_float());
    }

    #[test]
    fn test_is_numeric() {
        // Integers are numeric
        assert!(Type::Byte.is_numeric());
        assert!(Type::Word.is_numeric());
        assert!(Type::Sbyte.is_numeric());
        assert!(Type::Sword.is_numeric());
        // Fixed and float are numeric
        assert!(Type::Fixed.is_numeric());
        assert!(Type::Float.is_numeric());
        // Others are not
        assert!(!Type::Bool.is_numeric());
        assert!(!Type::String.is_numeric());
        assert!(!Type::Void.is_numeric());
    }

    #[test]
    fn test_is_signed() {
        assert!(!Type::Byte.is_signed());
        assert!(!Type::Word.is_signed());
        assert!(Type::Sbyte.is_signed());
        assert!(Type::Sword.is_signed());
        assert!(Type::Fixed.is_signed());
        assert!(Type::Float.is_signed());
        assert!(!Type::Bool.is_signed());
    }

    #[test]
    fn test_is_array() {
        assert!(!Type::Byte.is_array());
        assert!(!Type::Word.is_array());
        assert!(!Type::Bool.is_array());
        assert!(!Type::Sbyte.is_array());
        assert!(!Type::Sword.is_array());
        assert!(Type::ByteArray(Some(10)).is_array());
        assert!(Type::ByteArray(None).is_array());
        assert!(Type::WordArray(Some(10)).is_array());
        assert!(Type::WordArray(None).is_array());
        assert!(Type::BoolArray(Some(10)).is_array());
        assert!(Type::BoolArray(None).is_array());
        assert!(Type::SbyteArray(Some(10)).is_array());
        assert!(Type::SbyteArray(None).is_array());
        assert!(Type::SwordArray(Some(10)).is_array());
        assert!(Type::SwordArray(None).is_array());
    }

    #[test]
    fn test_element_type() {
        assert_eq!(Type::ByteArray(Some(10)).element_type(), Some(Type::Byte));
        assert_eq!(Type::WordArray(Some(10)).element_type(), Some(Type::Word));
        assert_eq!(Type::BoolArray(Some(10)).element_type(), Some(Type::Bool));
        assert_eq!(Type::SbyteArray(Some(10)).element_type(), Some(Type::Sbyte));
        assert_eq!(Type::SwordArray(Some(10)).element_type(), Some(Type::Sword));
        assert_eq!(Type::Byte.element_type(), None);
        assert_eq!(Type::String.element_type(), None);
    }

    #[test]
    fn test_assignable() {
        assert!(Type::Byte.is_assignable_to(&Type::Byte));
        assert!(Type::Byte.is_assignable_to(&Type::Word));
        assert!(Type::Byte.is_assignable_to(&Type::Sword));
        assert!(Type::Sbyte.is_assignable_to(&Type::Sword));
        assert!(!Type::Word.is_assignable_to(&Type::Byte));
        assert!(!Type::Sword.is_assignable_to(&Type::Sbyte));
    }

    #[test]
    fn test_assignable_to_fixed() {
        // All integers can be assigned to fixed
        assert!(Type::Byte.is_assignable_to(&Type::Fixed));
        assert!(Type::Sbyte.is_assignable_to(&Type::Fixed));
        assert!(Type::Word.is_assignable_to(&Type::Fixed));
        assert!(Type::Sword.is_assignable_to(&Type::Fixed));
        // Fixed to itself
        assert!(Type::Fixed.is_assignable_to(&Type::Fixed));
        // Fixed to float is allowed
        assert!(Type::Fixed.is_assignable_to(&Type::Float));
        // Float to fixed requires explicit cast
        assert!(!Type::Float.is_assignable_to(&Type::Fixed));
        // Fixed to integers requires explicit cast
        assert!(!Type::Fixed.is_assignable_to(&Type::Byte));
        assert!(!Type::Fixed.is_assignable_to(&Type::Sword));
    }

    #[test]
    fn test_assignable_to_float() {
        // All integers can be assigned to float
        assert!(Type::Byte.is_assignable_to(&Type::Float));
        assert!(Type::Sbyte.is_assignable_to(&Type::Float));
        assert!(Type::Word.is_assignable_to(&Type::Float));
        assert!(Type::Sword.is_assignable_to(&Type::Float));
        // Fixed can be assigned to float
        assert!(Type::Fixed.is_assignable_to(&Type::Float));
        // Float to itself
        assert!(Type::Float.is_assignable_to(&Type::Float));
        // Float to integers requires explicit cast
        assert!(!Type::Float.is_assignable_to(&Type::Byte));
        assert!(!Type::Float.is_assignable_to(&Type::Sword));
        // Float to fixed requires explicit cast
        assert!(!Type::Float.is_assignable_to(&Type::Fixed));
    }

    #[test]
    fn test_binary_result_type() {
        // Same types
        assert_eq!(
            Type::binary_result_type(&Type::Byte, &Type::Byte),
            Some(Type::Byte)
        );
        assert_eq!(
            Type::binary_result_type(&Type::Word, &Type::Word),
            Some(Type::Word)
        );
        assert_eq!(
            Type::binary_result_type(&Type::Bool, &Type::Bool),
            Some(Type::Bool)
        );

        // Mixed unsigned
        assert_eq!(
            Type::binary_result_type(&Type::Byte, &Type::Word),
            Some(Type::Word)
        );
        assert_eq!(
            Type::binary_result_type(&Type::Word, &Type::Byte),
            Some(Type::Word)
        );

        // Mixed signed
        assert_eq!(
            Type::binary_result_type(&Type::Sbyte, &Type::Sword),
            Some(Type::Sword)
        );

        // Invalid combinations
        assert_eq!(Type::binary_result_type(&Type::Byte, &Type::String), None);
        assert_eq!(Type::binary_result_type(&Type::Byte, &Type::Bool), None);
    }

    #[test]
    fn test_binary_result_type_fixed() {
        // Fixed + Fixed = Fixed
        assert_eq!(
            Type::binary_result_type(&Type::Fixed, &Type::Fixed),
            Some(Type::Fixed)
        );

        // Fixed + integer = Fixed
        assert_eq!(
            Type::binary_result_type(&Type::Fixed, &Type::Byte),
            Some(Type::Fixed)
        );
        assert_eq!(
            Type::binary_result_type(&Type::Sword, &Type::Fixed),
            Some(Type::Fixed)
        );

        // Fixed + Float = Float
        assert_eq!(
            Type::binary_result_type(&Type::Fixed, &Type::Float),
            Some(Type::Float)
        );

        // Fixed + non-numeric = None
        assert_eq!(Type::binary_result_type(&Type::Fixed, &Type::String), None);
        assert_eq!(Type::binary_result_type(&Type::Fixed, &Type::Bool), None);
    }

    #[test]
    fn test_binary_result_type_float() {
        // Float + Float = Float
        assert_eq!(
            Type::binary_result_type(&Type::Float, &Type::Float),
            Some(Type::Float)
        );

        // Float + integer = Float
        assert_eq!(
            Type::binary_result_type(&Type::Float, &Type::Byte),
            Some(Type::Float)
        );
        assert_eq!(
            Type::binary_result_type(&Type::Word, &Type::Float),
            Some(Type::Float)
        );

        // Float + Fixed = Float
        assert_eq!(
            Type::binary_result_type(&Type::Float, &Type::Fixed),
            Some(Type::Float)
        );

        // Float + non-numeric = None
        assert_eq!(Type::binary_result_type(&Type::Float, &Type::String), None);
        assert_eq!(Type::binary_result_type(&Type::Float, &Type::Bool), None);
    }

    #[test]
    fn test_type_name() {
        assert_eq!(Type::Byte.name(), "byte");
        assert_eq!(Type::Word.name(), "word");
        assert_eq!(Type::Sbyte.name(), "sbyte");
        assert_eq!(Type::Sword.name(), "sword");
        assert_eq!(Type::Fixed.name(), "fixed");
        assert_eq!(Type::Float.name(), "float");
        assert_eq!(Type::Bool.name(), "bool");
        assert_eq!(Type::String.name(), "string");
        assert_eq!(Type::Void.name(), "void");
        assert_eq!(Type::ByteArray(None).name(), "byte[]");
        assert_eq!(Type::WordArray(None).name(), "word[]");
        assert_eq!(Type::BoolArray(None).name(), "bool[]");
        assert_eq!(Type::SbyteArray(None).name(), "sbyte[]");
        assert_eq!(Type::SwordArray(None).name(), "sword[]");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Byte), "byte");
        assert_eq!(format!("{}", Type::Word), "word");
        assert_eq!(format!("{}", Type::Sbyte), "sbyte");
        assert_eq!(format!("{}", Type::Sword), "sword");
        assert_eq!(format!("{}", Type::Fixed), "fixed");
        assert_eq!(format!("{}", Type::Float), "float");
        assert_eq!(format!("{}", Type::Bool), "bool");
        assert_eq!(format!("{}", Type::String), "string");
        assert_eq!(format!("{}", Type::Void), "void");
        assert_eq!(format!("{}", Type::ByteArray(Some(10))), "byte[10]");
        assert_eq!(format!("{}", Type::ByteArray(None)), "byte[]");
        assert_eq!(format!("{}", Type::WordArray(Some(5))), "word[5]");
        assert_eq!(format!("{}", Type::WordArray(None)), "word[]");
        assert_eq!(format!("{}", Type::BoolArray(Some(8))), "bool[8]");
        assert_eq!(format!("{}", Type::BoolArray(None)), "bool[]");
        assert_eq!(format!("{}", Type::SbyteArray(Some(10))), "sbyte[10]");
        assert_eq!(format!("{}", Type::SbyteArray(None)), "sbyte[]");
        assert_eq!(format!("{}", Type::SwordArray(Some(5))), "sword[5]");
        assert_eq!(format!("{}", Type::SwordArray(None)), "sword[]");
    }

    #[test]
    fn test_type_equality() {
        assert_eq!(Type::Byte, Type::Byte);
        assert_ne!(Type::Byte, Type::Word);
        assert_eq!(Type::Fixed, Type::Fixed);
        assert_eq!(Type::Float, Type::Float);
        assert_ne!(Type::Fixed, Type::Float);
        assert_ne!(Type::Fixed, Type::Sword);
        assert_eq!(Type::ByteArray(Some(10)), Type::ByteArray(Some(10)));
        assert_ne!(Type::ByteArray(Some(10)), Type::ByteArray(Some(20)));
        assert_ne!(Type::ByteArray(Some(10)), Type::ByteArray(None));
        assert_eq!(Type::BoolArray(Some(10)), Type::BoolArray(Some(10)));
        assert_ne!(Type::BoolArray(Some(10)), Type::BoolArray(Some(20)));
        assert_ne!(Type::BoolArray(Some(10)), Type::ByteArray(Some(10)));
        // Signed array equality
        assert_eq!(Type::SbyteArray(Some(10)), Type::SbyteArray(Some(10)));
        assert_ne!(Type::SbyteArray(Some(10)), Type::SbyteArray(Some(20)));
        assert_ne!(Type::SbyteArray(Some(10)), Type::ByteArray(Some(10)));
        assert_eq!(Type::SwordArray(Some(10)), Type::SwordArray(Some(10)));
        assert_ne!(Type::SwordArray(Some(10)), Type::WordArray(Some(10)));
    }

    #[test]
    fn test_signed_array_assignable() {
        // Sized to unsized is allowed
        assert!(Type::SbyteArray(Some(10)).is_assignable_to(&Type::SbyteArray(None)));
        assert!(Type::SwordArray(Some(10)).is_assignable_to(&Type::SwordArray(None)));
        // Unsized to sized is not allowed
        assert!(!Type::SbyteArray(None).is_assignable_to(&Type::SbyteArray(Some(10))));
        assert!(!Type::SwordArray(None).is_assignable_to(&Type::SwordArray(Some(10))));
        // Different array types are not compatible
        assert!(!Type::SbyteArray(Some(10)).is_assignable_to(&Type::ByteArray(None)));
        assert!(!Type::ByteArray(Some(10)).is_assignable_to(&Type::SbyteArray(None)));
    }
}
