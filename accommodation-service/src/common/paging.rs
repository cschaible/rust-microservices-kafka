use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PageParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct Page<T: Serialize> {
    pub items: Vec<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_elements: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<usize>,
}
