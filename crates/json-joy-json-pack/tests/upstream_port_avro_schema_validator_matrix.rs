#![allow(clippy::approx_constant)]

use json_joy_json_pack::avro::{AvroField, AvroSchema, AvroSchemaValidator, AvroValue};

fn field(name: &str, type_: AvroSchema) -> AvroField {
    AvroField {
        name: name.to_string(),
        type_,
        default: None,
        doc: None,
        aliases: Vec::new(),
    }
}

fn field_with_default(name: &str, type_: AvroSchema, default: AvroValue) -> AvroField {
    AvroField {
        name: name.to_string(),
        type_,
        default: Some(default),
        doc: None,
        aliases: Vec::new(),
    }
}

#[test]
fn avro_schema_validator_schema_matrix() {
    let mut validator = AvroSchemaValidator::new();

    let primitive_refs = [
        "null", "boolean", "int", "long", "float", "double", "bytes", "string",
    ];
    for primitive in primitive_refs {
        assert!(validator.validate_schema(&AvroSchema::Ref(primitive.to_string())));
    }

    let primitive_objects = [
        AvroSchema::Null,
        AvroSchema::Boolean,
        AvroSchema::Int,
        AvroSchema::Long,
        AvroSchema::Float,
        AvroSchema::Double,
        AvroSchema::Bytes,
        AvroSchema::String,
    ];
    for schema in primitive_objects {
        assert!(validator.validate_schema(&schema));
    }

    assert!(validator.validate_schema(&AvroSchema::Record {
        name: "User".to_string(),
        namespace: None,
        fields: vec![
            field("id", AvroSchema::Int),
            field("name", AvroSchema::String)
        ],
        aliases: Vec::new(),
        doc: None,
    }));
    assert!(validator.validate_schema(&AvroSchema::Record {
        name: "User".to_string(),
        namespace: None,
        fields: vec![
            field("id", AvroSchema::Int),
            field_with_default(
                "name",
                AvroSchema::String,
                AvroValue::Str("Unknown".to_string())
            ),
        ],
        aliases: Vec::new(),
        doc: None,
    }));
    assert!(!validator.validate_schema(&AvroSchema::Record {
        name: String::new(),
        namespace: None,
        fields: vec![field("id", AvroSchema::Int)],
        aliases: Vec::new(),
        doc: None,
    }));
    assert!(!validator.validate_schema(&AvroSchema::Record {
        name: "User".to_string(),
        namespace: None,
        fields: vec![
            field("id", AvroSchema::Int),
            field("id", AvroSchema::String)
        ],
        aliases: Vec::new(),
        doc: None,
    }));

    assert!(validator.validate_schema(&AvroSchema::Enum {
        name: "Color".to_string(),
        namespace: None,
        symbols: vec!["RED".to_string(), "GREEN".to_string(), "BLUE".to_string()],
        default: None,
        aliases: Vec::new(),
    }));
    assert!(validator.validate_schema(&AvroSchema::Enum {
        name: "Color".to_string(),
        namespace: None,
        symbols: vec!["RED".to_string(), "GREEN".to_string(), "BLUE".to_string()],
        default: Some("RED".to_string()),
        aliases: Vec::new(),
    }));
    assert!(!validator.validate_schema(&AvroSchema::Enum {
        name: "Color".to_string(),
        namespace: None,
        symbols: Vec::new(),
        default: None,
        aliases: Vec::new(),
    }));
    assert!(!validator.validate_schema(&AvroSchema::Enum {
        name: "Color".to_string(),
        namespace: None,
        symbols: vec!["RED".to_string(), "GREEN".to_string(), "RED".to_string()],
        default: None,
        aliases: Vec::new(),
    }));
    assert!(!validator.validate_schema(&AvroSchema::Enum {
        name: "Color".to_string(),
        namespace: None,
        symbols: vec!["RED".to_string(), "GREEN".to_string(), "BLUE".to_string()],
        default: Some("YELLOW".to_string()),
        aliases: Vec::new(),
    }));

    assert!(validator.validate_schema(&AvroSchema::Array {
        items: Box::new(AvroSchema::String),
    }));
    assert!(validator.validate_schema(&AvroSchema::Array {
        items: Box::new(AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
        }),
    }));

    assert!(validator.validate_schema(&AvroSchema::Map {
        values: Box::new(AvroSchema::String),
    }));
    assert!(validator.validate_schema(&AvroSchema::Map {
        values: Box::new(AvroSchema::Record {
            name: "Value".to_string(),
            namespace: None,
            fields: vec![field("data", AvroSchema::String)],
            aliases: Vec::new(),
            doc: None,
        }),
    }));

    assert!(validator.validate_schema(&AvroSchema::Union(vec![
        AvroSchema::Ref("null".to_string()),
        AvroSchema::Ref("string".to_string()),
    ])));
    assert!(validator.validate_schema(&AvroSchema::Union(vec![
        AvroSchema::Ref("null".to_string()),
        AvroSchema::Ref("string".to_string()),
        AvroSchema::Record {
            name: "User".to_string(),
            namespace: None,
            fields: vec![field("id", AvroSchema::Int)],
            aliases: Vec::new(),
            doc: None,
        },
    ])));
    assert!(!validator.validate_schema(&AvroSchema::Union(Vec::new())));
    assert!(!validator.validate_schema(&AvroSchema::Union(vec![
        AvroSchema::Ref("string".to_string()),
        AvroSchema::Ref("string".to_string()),
    ])));

    assert!(validator.validate_schema(&AvroSchema::Fixed {
        name: "Hash".to_string(),
        namespace: None,
        size: 16,
        aliases: Vec::new(),
    }));
    assert!(!validator.validate_schema(&AvroSchema::Fixed {
        name: String::new(),
        namespace: None,
        size: 16,
        aliases: Vec::new(),
    }));

    assert!(validator.validate_schema(&AvroSchema::Record {
        name: "Node".to_string(),
        namespace: None,
        fields: vec![field(
            "next",
            AvroSchema::Union(vec![
                AvroSchema::Ref("null".to_string()),
                AvroSchema::Ref("Node".to_string()),
            ]),
        )],
        aliases: Vec::new(),
        doc: None,
    }));
}

#[test]
fn avro_schema_validator_value_matrix() {
    let mut validator = AvroSchemaValidator::new();

    assert!(validator.validate_value(&AvroValue::Null, &AvroSchema::Ref("null".to_string())));
    assert!(!validator.validate_value(
        &AvroValue::Str("x".to_string()),
        &AvroSchema::Ref("null".to_string())
    ));

    assert!(validator.validate_value(
        &AvroValue::Bool(true),
        &AvroSchema::Ref("boolean".to_string())
    ));
    assert!(!validator.validate_value(
        &AvroValue::Str("true".to_string()),
        &AvroSchema::Ref("boolean".to_string())
    ));

    assert!(validator.validate_value(&AvroValue::Int(42), &AvroSchema::Ref("int".to_string())));
    assert!(validator.validate_value(&AvroValue::Long(-42), &AvroSchema::Ref("int".to_string())));
    assert!(validator.validate_value(
        &AvroValue::Long(2_147_483_647),
        &AvroSchema::Ref("int".to_string())
    ));
    assert!(validator.validate_value(
        &AvroValue::Long(-2_147_483_648),
        &AvroSchema::Ref("int".to_string())
    ));
    assert!(!validator.validate_value(
        &AvroValue::Long(2_147_483_648),
        &AvroSchema::Ref("int".to_string())
    ));
    assert!(!validator.validate_value(
        &AvroValue::Double(3.14),
        &AvroSchema::Ref("int".to_string())
    ));

    assert!(validator.validate_value(&AvroValue::Int(42), &AvroSchema::Ref("long".to_string())));
    assert!(validator.validate_value(&AvroValue::Long(42), &AvroSchema::Ref("long".to_string())));
    assert!(!validator.validate_value(
        &AvroValue::Double(3.14),
        &AvroSchema::Ref("long".to_string())
    ));

    assert!(validator.validate_value(
        &AvroValue::Double(3.14),
        &AvroSchema::Ref("float".to_string())
    ));
    assert!(validator.validate_value(&AvroValue::Int(42), &AvroSchema::Ref("float".to_string())));
    assert!(validator.validate_value(
        &AvroValue::Double(3.14),
        &AvroSchema::Ref("double".to_string())
    ));
    assert!(!validator.validate_value(
        &AvroValue::Str("3.14".to_string()),
        &AvroSchema::Ref("float".to_string())
    ));

    assert!(validator.validate_value(
        &AvroValue::Bytes(vec![1, 2, 3]),
        &AvroSchema::Ref("bytes".to_string())
    ));
    assert!(!validator.validate_value(
        &AvroValue::Array(vec![AvroValue::Int(1), AvroValue::Int(2)]),
        &AvroSchema::Ref("bytes".to_string())
    ));

    assert!(validator.validate_value(
        &AvroValue::Str("hello".to_string()),
        &AvroSchema::Ref("string".to_string())
    ));
    assert!(validator.validate_value(
        &AvroValue::Str(String::new()),
        &AvroSchema::Ref("string".to_string())
    ));
    assert!(!validator.validate_value(&AvroValue::Int(42), &AvroSchema::Ref("string".to_string())));

    let user_schema = AvroSchema::Record {
        name: "User".to_string(),
        namespace: None,
        fields: vec![
            field("id", AvroSchema::Int),
            field("name", AvroSchema::String),
        ],
        aliases: Vec::new(),
        doc: None,
    };
    assert!(validator.validate_value(
        &AvroValue::Record(vec![
            ("id".to_string(), AvroValue::Int(1)),
            ("name".to_string(), AvroValue::Str("John".to_string())),
        ]),
        &user_schema
    ));
    assert!(!validator.validate_value(
        &AvroValue::Record(vec![("id".to_string(), AvroValue::Int(1))]),
        &user_schema
    ));
    assert!(!validator.validate_value(
        &AvroValue::Record(vec![
            ("id".to_string(), AvroValue::Str("1".to_string())),
            ("name".to_string(), AvroValue::Str("John".to_string())),
        ]),
        &user_schema
    ));

    let color_schema = AvroSchema::Enum {
        name: "Color".to_string(),
        namespace: None,
        symbols: vec!["RED".to_string(), "GREEN".to_string(), "BLUE".to_string()],
        default: None,
        aliases: Vec::new(),
    };
    assert!(validator.validate_value(&AvroValue::Enum("RED".to_string()), &color_schema));
    assert!(!validator.validate_value(&AvroValue::Enum("YELLOW".to_string()), &color_schema));
    assert!(!validator.validate_value(&AvroValue::Int(0), &color_schema));

    let array_schema = AvroSchema::Array {
        items: Box::new(AvroSchema::String),
    };
    assert!(validator.validate_value(
        &AvroValue::Array(vec![
            AvroValue::Str("a".to_string()),
            AvroValue::Str("b".to_string()),
            AvroValue::Str("c".to_string()),
        ]),
        &array_schema
    ));
    assert!(validator.validate_value(&AvroValue::Array(Vec::new()), &array_schema));
    assert!(!validator.validate_value(
        &AvroValue::Array(vec![
            AvroValue::Str("a".to_string()),
            AvroValue::Int(1),
            AvroValue::Str("c".to_string()),
        ]),
        &array_schema
    ));

    let map_schema = AvroSchema::Map {
        values: Box::new(AvroSchema::Int),
    };
    assert!(validator.validate_value(
        &AvroValue::Map(vec![
            ("a".to_string(), AvroValue::Int(1)),
            ("b".to_string(), AvroValue::Int(2)),
        ]),
        &map_schema
    ));
    assert!(validator.validate_value(&AvroValue::Map(Vec::new()), &map_schema));
    assert!(!validator.validate_value(
        &AvroValue::Map(vec![
            ("a".to_string(), AvroValue::Int(1)),
            ("b".to_string(), AvroValue::Str("two".to_string())),
        ]),
        &map_schema
    ));

    let union_schema = AvroSchema::Union(vec![
        AvroSchema::Ref("null".to_string()),
        AvroSchema::Ref("string".to_string()),
        AvroSchema::Ref("int".to_string()),
    ]);
    assert!(validator.validate_value(&AvroValue::Null, &union_schema));
    assert!(validator.validate_value(&AvroValue::Str("hello".to_string()), &union_schema));
    assert!(validator.validate_value(&AvroValue::Int(42), &union_schema));
    assert!(!validator.validate_value(&AvroValue::Double(3.14), &union_schema));

    let fixed_schema = AvroSchema::Fixed {
        name: "Hash".to_string(),
        namespace: None,
        size: 4,
        aliases: Vec::new(),
    };
    assert!(validator.validate_value(&AvroValue::Fixed(vec![1, 2, 3, 4]), &fixed_schema));
    assert!(!validator.validate_value(&AvroValue::Fixed(vec![1, 2, 3]), &fixed_schema));
    assert!(!validator.validate_value(&AvroValue::Fixed(vec![1, 2, 3, 4, 5]), &fixed_schema));
}
