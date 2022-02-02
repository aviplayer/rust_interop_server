use crate::Handler;

pub struct UriHandler{
    pub uri: String,
    pub handler: Option<Box<dyn Handler>>
}

