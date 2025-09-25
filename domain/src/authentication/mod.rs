use tracing::instrument;

pub mod archived_authenticated_user;
pub mod authenticated_user;
pub mod user_authentication_request;

#[instrument(level = "trace")]
pub fn create_class_ids() -> Vec<String> {
    let years = ["1", "2", "3", "4"];
    let classes = ["a", "b", "c", "g", "ga", "gb", "k"];
    let mut class_ids = Vec::new();
    for year in years.iter() {
        for class in classes.iter() {
            let class_id = format!("{}{}", year, class);
            class_ids.push(class_id);
        }
    }
    class_ids
}
