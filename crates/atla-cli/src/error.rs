#[derive(Debug, thiserror::Error)]
pub enum AtlaCliError {
    #[error("{0} is not implemented yet")]
    NotImplemented(&'static str),
}
