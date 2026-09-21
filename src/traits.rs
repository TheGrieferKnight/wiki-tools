pub trait Searchable {
    fn matches(&self, search_field: &crate::records::SearchField) -> bool;
}
