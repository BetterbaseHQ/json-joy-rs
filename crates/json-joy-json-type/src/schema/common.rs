/// Properties that are used to display to the user.
/// Upstream reference: json-type/src/schema/common.ts
#[derive(Debug, Clone, Default)]
pub struct Display {
    pub title: Option<String>,
    pub intro: Option<String>,
    pub description: Option<String>,
}
