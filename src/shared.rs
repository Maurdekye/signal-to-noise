use crate::Args;
use ggez_no_re::csv_recorder::CsvRecorder;

#[derive(Clone)]
pub struct Shared {
    pub args: Args,
    pub recorder: CsvRecorder,
}

impl Shared {
    pub fn new(args: Args) -> Shared {
        let recorder = CsvRecorder::new(&args.record_path);
        Shared { args, recorder }
    }
}
