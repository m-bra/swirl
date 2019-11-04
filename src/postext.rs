    
/// pos is not an index in 'text', it marks the global position of text[0]
#[derive(Clone)]
pub struct PosText {
    pub text: String,
    pub pos: usize,
}

impl PosText {
    // go forward by 'offset' bytes
    pub fn forward(&self, offset: usize) -> PosText {
        PosText {
            text: self.text[offset..].to_string(),
            pos: self.pos + offset,
        }
    }
}
