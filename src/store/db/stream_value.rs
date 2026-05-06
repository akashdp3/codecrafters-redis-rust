#[derive(Debug)]
pub(crate) struct StreamValue {
    id: String,
    fields: Vec<(String, String)>,
}

impl StreamValue {
    pub(super) fn new(id: String, fields: Vec<(String, String)>) -> Self {
        Self { id, fields }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn fields(&self) -> &[(String, String)] {
        &self.fields
    }
}
