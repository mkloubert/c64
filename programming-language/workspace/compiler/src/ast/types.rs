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
    /// Boolean value.
    Bool,
    /// Text string.
    String,
    /// Byte array.
    ByteArray(Option<u16>),
    /// Word array.
    WordArray(Option<u16>),
    /// Void (no value, for functions).
    Void,
}

impl Type {
    /// Get the size of this type in bytes.
    pub fn size(&self) -> usize {
        match self {
            Type::Byte | Type::Sbyte | Type::Bool => 1,
            Type::Word | Type::Sword => 2,
            Type::String => 256, // Maximum string size
            Type::ByteArray(Some(n)) => *n as usize,
            Type::ByteArray(None) => 0, // Unknown size
            Type::WordArray(Some(n)) => (*n as usize) * 2,
            Type::WordArray(None) => 0, // Unknown size
            Type::Void => 0,
        }
    }

    /// Check if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Byte | Type::Word | Type::Sbyte | Type::Sword)
    }

    /// Check if this is a signed type.
    pub fn is_signed(&self) -> bool {
        matches!(self, Type::Sbyte | Type::Sword)
    }

    /// Check if this is an array type.
    pub fn is_array(&self) -> bool {
        matches!(self, Type::ByteArray(_) | Type::WordArray(_))
    }

    /// Get the element type if this is an array.
    pub fn element_type(&self) -> Option<Type> {
        match self {
            Type::ByteArray(_) => Some(Type::Byte),
            Type::WordArray(_) => Some(Type::Word),
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
            (Type::Byte, Type::Word) => true,
            (Type::Byte, Type::Sword) => true,
            (Type::Sbyte, Type::Sword) => true,
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
            (Type::Bool, Type::Bool) => Some(Type::Bool),

            // Mixed unsigned: promote to larger
            (Type::Byte, Type::Word) | (Type::Word, Type::Byte) => Some(Type::Word),

            // Mixed signed: promote to larger
            (Type::Sbyte, Type::Sword) | (Type::Sword, Type::Sbyte) => Some(Type::Sword),

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
            Type::Bool => "bool",
            Type::String => "string",
            Type::ByteArray(_) => "byte[]",
            Type::WordArray(_) => "word[]",
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
        assert_eq!(Type::String.size(), 256);
        assert_eq!(Type::Void.size(), 0);
    }

    #[test]
    fn test_array_size() {
        assert_eq!(Type::ByteArray(Some(10)).size(), 10);
        assert_eq!(Type::ByteArray(None).size(), 0);
        assert_eq!(Type::WordArray(Some(10)).size(), 20);
        assert_eq!(Type::WordArray(None).size(), 0);
    }

    #[test]
    fn test_is_integer() {
        assert!(Type::Byte.is_integer());
        assert!(Type::Word.is_integer());
        assert!(Type::Sbyte.is_integer());
        assert!(Type::Sword.is_integer());
        assert!(!Type::Bool.is_integer());
        assert!(!Type::String.is_integer());
        assert!(!Type::Void.is_integer());
    }

    #[test]
    fn test_is_signed() {
        assert!(!Type::Byte.is_signed());
        assert!(!Type::Word.is_signed());
        assert!(Type::Sbyte.is_signed());
        assert!(Type::Sword.is_signed());
        assert!(!Type::Bool.is_signed());
    }

    #[test]
    fn test_is_array() {
        assert!(!Type::Byte.is_array());
        assert!(!Type::Word.is_array());
        assert!(Type::ByteArray(Some(10)).is_array());
        assert!(Type::ByteArray(None).is_array());
        assert!(Type::WordArray(Some(10)).is_array());
        assert!(Type::WordArray(None).is_array());
    }

    #[test]
    fn test_element_type() {
        assert_eq!(Type::ByteArray(Some(10)).element_type(), Some(Type::Byte));
        assert_eq!(Type::WordArray(Some(10)).element_type(), Some(Type::Word));
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
    fn test_type_name() {
        assert_eq!(Type::Byte.name(), "byte");
        assert_eq!(Type::Word.name(), "word");
        assert_eq!(Type::Sbyte.name(), "sbyte");
        assert_eq!(Type::Sword.name(), "sword");
        assert_eq!(Type::Bool.name(), "bool");
        assert_eq!(Type::String.name(), "string");
        assert_eq!(Type::Void.name(), "void");
        assert_eq!(Type::ByteArray(None).name(), "byte[]");
        assert_eq!(Type::WordArray(None).name(), "word[]");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Byte), "byte");
        assert_eq!(format!("{}", Type::Word), "word");
        assert_eq!(format!("{}", Type::Sbyte), "sbyte");
        assert_eq!(format!("{}", Type::Sword), "sword");
        assert_eq!(format!("{}", Type::Bool), "bool");
        assert_eq!(format!("{}", Type::String), "string");
        assert_eq!(format!("{}", Type::Void), "void");
        assert_eq!(format!("{}", Type::ByteArray(Some(10))), "byte[10]");
        assert_eq!(format!("{}", Type::ByteArray(None)), "byte[]");
        assert_eq!(format!("{}", Type::WordArray(Some(5))), "word[5]");
        assert_eq!(format!("{}", Type::WordArray(None)), "word[]");
    }

    #[test]
    fn test_type_equality() {
        assert_eq!(Type::Byte, Type::Byte);
        assert_ne!(Type::Byte, Type::Word);
        assert_eq!(Type::ByteArray(Some(10)), Type::ByteArray(Some(10)));
        assert_ne!(Type::ByteArray(Some(10)), Type::ByteArray(Some(20)));
        assert_ne!(Type::ByteArray(Some(10)), Type::ByteArray(None));
    }
}
