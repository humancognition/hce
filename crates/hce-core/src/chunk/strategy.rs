use crate::chunk::ChunkSpec;
use crate::token::Token;

pub trait ChunkStrategy: Send + Sync {
    fn boundaries(&self, tokens: &[Token], spec: &ChunkSpec) -> alloc::vec::Vec<usize>;
}
