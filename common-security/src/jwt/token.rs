use std::sync::Arc;
use serde::Deserialize;

use crate::jwt::error::TokenDecoderError;
use crate::jwt::Claims;
use crate::jwt::default_jwt::DefaultJwt;

/// Token decoder claim trait definition. Decodes a string token to either a
/// boxed instance of `Claims` or returns an error.
pub trait TokenDecoder<T: for<'b> Deserialize<'b> + Claims>: TokenDecoderClone<T> {
    fn decode_token(&self, token: &str) -> Result<Box<T>, TokenDecoderError>;
}

/// A token decoder must be cloneable, `send` and `sync`.
/// Therefore it has to implement the `TokenDecoderClone` trait to be cloneable
/// as a boxed object.
pub trait TokenDecoderClone<T: for<'b> Deserialize<'b> + Claims>: Send + Sync {
    fn clone_box(&self) -> Box<dyn TokenDecoder<T>>;
}

impl<T: for<'b> Deserialize<'b> + Claims, U> TokenDecoderClone<T> for U
where U: 'static + TokenDecoder<T> + Clone
{
    fn clone_box(&self) -> Box<dyn TokenDecoder<T>> {
        Box::new(self.clone())
    }
}

impl<T: for<'b> Deserialize<'b> + Claims> Clone for Box<dyn TokenDecoder<T>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub struct TokenDecoders {
    // TODO make more flexible (replace DefaultJwt with T: for<'b> Deserialize<'b> + Claims
    pub decoders: Vec<Box<dyn TokenDecoder<DefaultJwt>>>
}

pub type DynTokenDecoders = Arc<TokenDecoders>;
