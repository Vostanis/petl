pub mod extract;
pub mod load;

pub trait Http: extract::HttpExtractExt {}
impl<T: extract::HttpExtractExt> Http for T {}
