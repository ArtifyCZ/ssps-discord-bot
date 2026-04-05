use tracing::instrument;

pub mod archived_authenticated_user;
pub mod authenticated_user;
pub mod user_authentication_request;

#[instrument(level = "trace")]
pub fn create_class_ids() -> Vec<String> {
    let prefixes = ["", "c", "h", "g", "l"];
    let years = ["1", "2", "3", "4"];
    let suffixes = ["a", "b", "c", "d", "g", "ga", "gb", "k"];
    let mut class_ids = Vec::new();
    for prefix in prefixes.iter() {
        for year in years.iter() {
            for suffix in suffixes.iter() {
                let class_id = format!("{}{}{}", prefix, year, suffix);
                class_ids.push(class_id);
            }
        }
    }
    class_ids
}
