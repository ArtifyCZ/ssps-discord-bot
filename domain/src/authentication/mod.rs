use tracing::instrument;

pub mod archived_authenticated_user;
pub mod authenticated_user;
pub mod user_authentication_request;

#[instrument(level = "trace")]
pub fn create_class_ids() -> Vec<String> {
    vec![
        "1A", "1B", "1C", "1GA", "1GB", "1GC", "1GD", "1K", "2A", "2B", "2C", "2GA", "2GB", "2K",
        "3A", "3B", "3C", "3GA", "3GB", "3K", "4A", "4B", "4C", "4G", "4K", "C1A", "C2A", "C2B",
        "C3A", "C3B", "C3C", "C4A", "C4B", "C4C", "G2A", "G3A", "G3B", "H1A", "H1B", "H2A", "H2B",
        "H3A", "H3B", "H3C", "H3D", "H4A", "L1A", "L1B", "L2A",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}
