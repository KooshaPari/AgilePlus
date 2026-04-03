//! # Phenotype Macros
//!
//! Utility macros for the Phenotype ecosystem.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

/// Create a newtype wrapper with basic implementations
///
/// # Example
/// ```rust
/// use phenotype_macros::newtype;
///
/// newtype!(pub struct UserId(String));
///
/// let id = UserId::new("user123");
/// assert_eq!(id.value(), "user123");
/// ```
#[macro_export]
macro_rules! newtype {
    ($(#[$meta:meta])* $vis:vis struct $name:ident($inner:ty)) => {
        $(#[$meta])*
        $vis struct $name(pub $inner);

        impl $name {
            /// Create a new instance
            #[must_use]
            pub fn new(value: $inner) -> Self {
                Self(value)
            }

            /// Get the inner value
            #[must_use]
            pub fn value(&self) -> &$inner {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

/// Create an enum with common variant patterns
#[macro_export]
macro_rules! enum_delegate {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $({
                    $($field:ident: $field_ty:ty),* $(,)?
                })?
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant $({
                    $($field: $field_ty),*
                })?
            ),*
        }

        impl $name {
            /// Get the variant name as a string
            #[must_use]
            pub fn variant_name(&self) -> &'static str {
                match self {
                    $(Self::$variant $({ $($field),* })?) => stringify!($variant),)*
                }
            }

            /// Check if this is a specific variant
            #[must_use]
            pub fn is(&self, other: &Self) -> bool {
                std::mem::discriminant(self) == std::mem::discriminant(other)
            }
        }
    };
}

/// Create a builder pattern struct
#[macro_export]
macro_rules! builder {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $field_ty:ty $(= $default:expr)?
            ),* $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $field_ty
            ),*
        }

        impl $name {
            /// Create a new builder with default values
            #[must_use]
            pub fn builder() -> $name {
                $name {
                    $($field: $($default)? .into(),)*
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::builder()
            }
        }
    };
}

/// Create a result type with a specific error
#[macro_export]
macro_rules! result_type {
    (
        $(#[$meta:meta])*
        $vis:vis type $name:ident = Result<T, $error:ident>;
    ) => {
        $(#[$meta])*
        $vis type $name = std::result::Result<(), $error>;
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtype_macro() {
        newtype!(pub struct TestId(String));

        let id = TestId::new("test123".to_string());
        assert_eq!(id.value(), "test123");
        assert_eq!(id.to_string(), "test123");
    }

    #[test]
    fn test_newtype_from() {
        newtype!(struct Count(i32));

        let count = Count::from(42);
        assert_eq!(count.value(), &42);
    }

    #[test]
    fn test_newtype_debug() {
        newtype!(struct DebugId(String));

        let id = DebugId::new("debug".to_string());
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("DebugId"));
    }

    #[test]
    fn test_builder_macro() {
        builder!(
            #[derive(Default)]
            struct TestBuilder {
                name: String = String::new(),
                count: i32 = 0,
            }
        );

        let builder = TestBuilder::builder();
        assert_eq!(builder.name, "");
        assert_eq!(builder.count, 0);

        let custom = TestBuilder {
            name: "custom".to_string(),
            count: 42,
        };
        assert_eq!(custom.name, "custom");
        assert_eq!(custom.count, 42);
    }
}
