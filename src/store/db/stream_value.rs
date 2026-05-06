#[derive(Debug)]
pub(crate) struct StreamValue {
    pub(crate) id: String,
    fields: Vec<(String, String)>,
}

impl StreamValue {
    pub(super) fn new(id: String, fields: Vec<(String, String)>) -> Self {
        Self { id, fields }
    }

    pub(super) fn _id(&self) -> &str {
        &self.id
    }

    pub(super) fn _fields(&self) -> &[(String, String)] {
        &self.fields
    }
}
