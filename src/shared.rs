use crate::{Args, recorder::Recorder};

#[derive(Clone)]
pub struct Shared {
    pub args: Args,
    pub recorder: Recorder,
}

impl Shared {
    pub fn new(args: Args) -> Shared {
        let recorder = Recorder::new(&args);
        Shared { args, recorder }
    }
}
