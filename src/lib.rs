use serde::{self, Deserialize, Serialize};
use std::str::Chars;

pub struct TransJson<'a> {
    s: &'a str,
}

impl<'a> TransJson<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s: s }
    }
    pub fn trans(&self) -> String {
        TransJson::escape_base(&TransJson::oneline_base(self.s))
    }
    pub fn oneline(&self) -> String {
        TransJson::oneline_base(self.s)
    }
    pub fn escape(&self) -> String {
        TransJson::escape_base(self.s)
    }
    fn oneline_base(s: &str) -> String {
        let mut will_oneline = s.to_string();
        will_oneline.retain(|c| c != '\r');
        will_oneline.retain(|c| c != '\n');
        will_oneline
    }
    fn escape_base(s: &str) -> String {
        escape_for_nested_json(s)
    }
}

fn trans_with_level(
    level: u32,
    next: bool,
    is_not_array: bool,
    json_chars: &mut Chars<'_>,
) -> String {
    // 例えば'{'の時：'"{'という風にダブルクォーテーションをつけ、再帰
    let mut text = "".to_string();
    // level 0 : 2^0-1 : "{"
    // level 1 : 2^1-1 : "\{"
    // level 2 : 2^2-1 : "\\\{"
    // level 3 : 2^3-1 : "\\\\\\\{"
    // 2^level - 1
    let escape = "\\".repeat((1 << level) - 1);
    while let Some(ch) = json_chars.next() {
        match ch {
            '"' => {
                text.push_str(&escape);
                text.push('"');
            }
            '{' => {
                if next {
                    text.push_str(&escape);
                    text.push('"');
                }
                text.push('{');
                text.push_str(&trans_with_level(
                    level + if is_not_array { 1 } else { 0 },
                    true,
                    true,
                    json_chars,
                ));
                if next {
                    text.push_str(&escape);
                    text.push('"');
                }
            }
            '}' => {
                text.push('}');
                return text;
            }
            '[' => {
                text.push_str(&escape);
                text.push('"');
                text.push('[');
                text.push_str(&trans_with_level(level, false, true, json_chars));
            }
            ']' => {
                text.push(']');
                if is_not_array {
                    text.push_str(&escape);
                    text.push('"');
                }
                return text;
            }
            any => {
                text.push(any);
            }
        }
    }
    unreachable!();
}
fn escape_for_nested_json(s: &str) -> String {
    let buffer = s.to_string();
    let json_chars: &mut Chars<'_> = &mut buffer.chars();
    let mut text = "".to_string();
    while let Some(ch) = json_chars.next() {
        match ch {
            '{' => {
                text.push('{');
                text.push_str(&trans_with_level(0, true, true, json_chars));
            }
            '[' => {
                text.push('[');
                text.push_str(&trans_with_level(0, false, false, json_chars));
            }
            any => {
                text.push(any);
            }
        }
    }
    text
}

// fn main() {
//     tests::small();
//     tests::big();
//     tests::array();
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct A {
        #[serde(with = "serde_with::json::nested")]
        other_struct: B,
    }
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct B {
        value: i32,
    }
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct C {
        #[serde(with = "serde_with::json::nested")]
        other_struct: D,
    }
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct D {
        #[serde(with = "serde_with::json::nested")]
        other_structs: Vec<B>,
    }

    #[test]
    pub fn small() {
        let x = A {
            other_struct: B { value: 42 },
        };

        // one line and use escape
        let s: &str = &r#"{"other_struct":"{\"value\":42}"}"#;
        let json = s;
        // println!("{}", json);
        let ok: A = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);

        // one line and not use escape
        let s: &str = &r#"{"other_struct":{"value":42}}"#;
        let translator = TransJson::new(s);
        let json = translator.escape();
        // println!("{}", json);
        let ok: A = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);

        // multi line and use escape
        let s: &str = &r#"
        {
            "other_struct": "{
                \"value\": 42
            }"
        }
        "#;
        let translator = TransJson::new(s);
        let json = translator.oneline();
        // println!("{}", json);
        let ok: A = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);

        // multi line and not use escape
        let s: &str = &r#"
        {
            "other_struct": {
                "value": 42
            }
        }
        "#;
        let translator = TransJson::new(s);
        let json = translator.trans();
        // println!("{}", json);
        let ok: A = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);
    }

    #[test]
    pub fn big() {
        let x = C {
            other_struct: D {
                other_structs: vec![B { value: 42 }],
            },
        };

        // one line and use escape
        let s: &str = r#"{"other_struct":"{\"other_structs\":\"[{\\\"value\\\":42}]\"}"}"#;
        let json = s;
        // println!("{}", json);
        let ok: C = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);

        // one line and not use escape
        let s = r#"{"other_struct":{"other_structs":[{"value":42}]}}"#;
        let translator = TransJson::new(s);
        let json = translator.escape();
        // println!("{}", json);
        let ok: C = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);

        // multi line and use escape
        let s = r#"
        {
            "other_struct" : "{
                \"other_structs\" : \"[
                    {
                        \\\"value\\\" : 42
                    }
                ]\"
            }"
        }
        "#;
        let translator = TransJson::new(s);
        let json = translator.oneline();
        // println!("{}", json);
        let ok: C = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);

        // multi line and not use escape
        let s = r#"
        {
            "other_struct" : {
                "other_structs" : [
                    {
                        "value" : 42
                    }
                ]
            }
        }
        "#;
        let translator = TransJson::new(s);
        let json = translator.trans();
        // println!("{}", json);
        let ok: C = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);
    }
    #[test]
    pub fn array() {
        let x = vec![B { value: 42 }, B { value: 28 }];
        let s = r#"
        [
            {
                "value": 42
            },
            {
                "value": 28
            }
        ]
        "#;
        let translator = TransJson::new(s);
        let json = translator.trans();
        // println!("{}", json);
        let ok: Vec<B> = serde_json::from_str(&json).unwrap();
        assert_eq!(ok, x);
    }
}
