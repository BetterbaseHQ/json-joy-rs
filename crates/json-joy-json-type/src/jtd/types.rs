//! JTD form types.
//!
//! Upstream reference: json-type/src/jtd/types.ts

use std::collections::HashMap;

/// A JSON Type Definition scalar type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JtdType {
    Boolean,
    Float32,
    Float64,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    String,
    Timestamp,
}

impl JtdType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Float32 => "float32",
            Self::Float64 => "float64",
            Self::Int8 => "int8",
            Self::Uint8 => "uint8",
            Self::Int16 => "int16",
            Self::Uint16 => "uint16",
            Self::Int32 => "int32",
            Self::Uint32 => "uint32",
            Self::String => "string",
            Self::Timestamp => "timestamp",
        }
    }
}

/// A JTD form — one of the eight JTD form kinds.
#[derive(Debug, Clone)]
pub enum JtdForm {
    /// Empty form: `{}` or `{nullable: bool}`
    Empty { nullable: bool },
    /// Ref form: `{ref: "TypeName"}`
    Ref { ref_: String },
    /// Type form: `{type: JtdType}`
    Type { type_: JtdType },
    /// Enum form: `{enum: [...]}`
    Enum { variants: Vec<String> },
    /// Elements form: `{elements: JtdForm}`
    Elements { elements: Box<JtdForm> },
    /// Properties form: `{properties: {...}, optionalProperties: {...}}`
    Properties {
        properties: HashMap<String, JtdForm>,
        optional_properties: HashMap<String, JtdForm>,
        additional_properties: bool,
    },
    /// Values form: `{values: JtdForm}`
    Values { values: Box<JtdForm> },
    /// Discriminator form: `{discriminator: "field", mapping: {...}}`
    Discriminator {
        discriminator: String,
        mapping: HashMap<String, JtdForm>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jtd_type_as_str_returns_correct_strings() {
        assert_eq!(JtdType::Boolean.as_str(), "boolean");
        assert_eq!(JtdType::Float32.as_str(), "float32");
        assert_eq!(JtdType::Float64.as_str(), "float64");
        assert_eq!(JtdType::Int8.as_str(), "int8");
        assert_eq!(JtdType::Uint8.as_str(), "uint8");
        assert_eq!(JtdType::Int16.as_str(), "int16");
        assert_eq!(JtdType::Uint16.as_str(), "uint16");
        assert_eq!(JtdType::Int32.as_str(), "int32");
        assert_eq!(JtdType::Uint32.as_str(), "uint32");
        assert_eq!(JtdType::String.as_str(), "string");
        assert_eq!(JtdType::Timestamp.as_str(), "timestamp");
    }

    #[test]
    fn jtd_type_clone_and_eq() {
        let t = JtdType::Int32;
        let t2 = t.clone();
        assert_eq!(t, t2);
        assert_ne!(JtdType::Int32, JtdType::Uint32);
    }

    #[test]
    fn jtd_type_debug_format() {
        let s = format!("{:?}", JtdType::Boolean);
        assert_eq!(s, "Boolean");
    }

    #[test]
    fn jtd_form_empty() {
        let form = JtdForm::Empty { nullable: false };
        // Verify it can be cloned and debugged
        let form2 = form.clone();
        let _ = format!("{:?}", form2);
    }

    #[test]
    fn jtd_form_ref() {
        let form = JtdForm::Ref {
            ref_: "MyType".to_string(),
        };
        if let JtdForm::Ref { ref_ } = &form {
            assert_eq!(ref_, "MyType");
        } else {
            panic!("Expected Ref variant");
        }
    }

    #[test]
    fn jtd_form_type() {
        let form = JtdForm::Type {
            type_: JtdType::String,
        };
        if let JtdForm::Type { type_ } = &form {
            assert_eq!(type_.as_str(), "string");
        } else {
            panic!("Expected Type variant");
        }
    }

    #[test]
    fn jtd_form_enum() {
        let form = JtdForm::Enum {
            variants: vec!["a".into(), "b".into(), "c".into()],
        };
        if let JtdForm::Enum { variants } = &form {
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0], "a");
        } else {
            panic!("Expected Enum variant");
        }
    }

    #[test]
    fn jtd_form_elements() {
        let inner = JtdForm::Type {
            type_: JtdType::Int32,
        };
        let form = JtdForm::Elements {
            elements: Box::new(inner),
        };
        if let JtdForm::Elements { elements } = &form {
            if let JtdForm::Type { type_ } = elements.as_ref() {
                assert_eq!(*type_, JtdType::Int32);
            } else {
                panic!("Expected inner Type");
            }
        } else {
            panic!("Expected Elements variant");
        }
    }

    #[test]
    fn jtd_form_properties() {
        let mut props = HashMap::new();
        props.insert(
            "name".to_string(),
            JtdForm::Type {
                type_: JtdType::String,
            },
        );
        let form = JtdForm::Properties {
            properties: props,
            optional_properties: HashMap::new(),
            additional_properties: false,
        };
        if let JtdForm::Properties {
            properties,
            optional_properties,
            additional_properties,
        } = &form
        {
            assert_eq!(properties.len(), 1);
            assert!(properties.contains_key("name"));
            assert!(optional_properties.is_empty());
            assert!(!additional_properties);
        } else {
            panic!("Expected Properties variant");
        }
    }

    #[test]
    fn jtd_form_values() {
        let form = JtdForm::Values {
            values: Box::new(JtdForm::Type {
                type_: JtdType::Boolean,
            }),
        };
        if let JtdForm::Values { values } = &form {
            if let JtdForm::Type { type_ } = values.as_ref() {
                assert_eq!(*type_, JtdType::Boolean);
            } else {
                panic!("Expected inner Type");
            }
        } else {
            panic!("Expected Values variant");
        }
    }

    #[test]
    fn jtd_form_discriminator() {
        let mut mapping = HashMap::new();
        mapping.insert("user".to_string(), JtdForm::Empty { nullable: false });
        let form = JtdForm::Discriminator {
            discriminator: "type".to_string(),
            mapping,
        };
        if let JtdForm::Discriminator {
            discriminator,
            mapping,
        } = &form
        {
            assert_eq!(discriminator, "type");
            assert_eq!(mapping.len(), 1);
        } else {
            panic!("Expected Discriminator variant");
        }
    }
}
