use crate::prelude::*;

pub struct Data {
    pub label: String,
    pub file: Option<dioxus::html::FileData>,
}

impl dioxus::html::HasFileData for Data {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        self.file.clone().into_iter().collect()
    }
}

impl HasFormData for Data {
    fn valid(&self) -> bool {
        true
    }
    fn value(&self) -> String {
        panic!("This should never be called, as we are using a custom form data handler");
    }
    fn values(&self) -> Vec<(String, FormValue)> {
        vec![
            ("label".to_string(), FormValue::Text(self.label.clone())),
            ("file".to_string(), FormValue::File(self.file.clone())),
        ]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
