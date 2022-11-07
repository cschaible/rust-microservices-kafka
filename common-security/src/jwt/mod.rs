use downcast_rs::impl_downcast;
use downcast_rs::Downcast;

pub mod default_jwt;
pub mod error;
pub mod rsa;
pub mod token;

pub trait Claims: Downcast + Sync + Send {}
impl_downcast!(Claims);
