use tracing::instrument;

pub mod authenticated_user;
pub mod user_authentication_request;

#[instrument(level = "trace")]
pub fn create_class_user_group_id_mails() -> Vec<(String, String)> {
    let years = ["1", "2", "3", "4"];
    let classes = ["a", "b", "c", "g", "ga", "gb", "k"];
    let mut user_group_mails = Vec::new();
    for year in years.iter() {
        for class in classes.iter() {
            let class_id = format!("{}{}", year, class);
            let mail = format!("{}@ssps.cz", class_id);
            user_group_mails.push((class_id, mail));
        }
    }
    user_group_mails
}
