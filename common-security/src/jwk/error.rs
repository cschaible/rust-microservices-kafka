#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum JwkLoaderError {
    /// The keyfile cannot be read from the local file system.
    KeyFileCouldNotBeRead,

    /// The keyfile is invalid.
    InvalidKeyFile,

    /// The downloaded JWKs are invalid.
    InvalidJsonResponse,

    /// The JWKs couldn't be downloaded.
    JwkDownloadFailed,
}
