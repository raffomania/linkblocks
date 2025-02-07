use serde::Serialize;
use serde_json::Value;

pub fn assert_form_matches<I: Serialize>(form: &visdom::types::Elements, input: &I) {
  let json_input = serde_json::to_value(input).unwrap();
  let json_input = json_input.as_object().unwrap();
  for field in form.find("input") {
    let html = field.outer_html();
    tracing::debug!("Checking form field {html}");
    let value = json_input.get(&field.get_attribute("name").unwrap().to_string());

    let Some(value) = value else {
      assert!(
        field.get_attribute("required").is_some(),
        "Missing value for required form field {html}"
      );
      continue;
    };
    tracing::debug!("Found input value: {value}");

    let field_type = field.get_attribute("type").unwrap().to_string();

    let types_match = matches!(
      (field_type.as_str(), value),
      ("text" | "password", Value::String(_))
        | ("number", Value::Number(_))
        | ("checkbox", Value::Bool(_))
    );

    assert!(
      types_match,
      r#"Input type "{field_type}" and value {value} don't match!"#
    );
  }
}
