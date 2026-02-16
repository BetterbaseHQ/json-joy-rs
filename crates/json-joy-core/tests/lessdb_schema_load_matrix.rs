use json_joy_core::less_db_compat::{
    create_model_with_schema, model_from_binary, model_load_with_schema, model_to_binary,
    view_model,
};
use json_joy_core::schema;
use serde_json::json;

#[test]
fn lessdb_schema_create_seeds_new_doc_matrix() {
    let sid = 881_001;
    let schema = schema::obj_node(vec![("doc".into(), schema::str_node("init"))], Vec::new());
    let model = create_model_with_schema(&schema, sid, true).expect("create with schema");
    assert_eq!(view_model(&model), json!({"doc":"init"}));
}

#[test]
fn lessdb_schema_load_skips_non_empty_matrix() {
    let sid = 881_002;
    let seed = create_model_with_schema(
        &schema::obj_node(vec![("x".into(), schema::con_json(json!(1)))], Vec::new()),
        sid,
        false,
    )
    .expect("seed model");
    let base = model_from_binary(&model_to_binary(&seed)).expect("parse base");
    let base_bin = model_to_binary(&base);

    let override_schema = schema::obj_node(
        vec![("doc".into(), schema::str_node("ignored"))],
        Vec::new(),
    );
    let loaded =
        model_load_with_schema(&base_bin, sid, &override_schema, true).expect("load with schema");
    assert_eq!(view_model(&loaded), json!({"x":1}));
}
