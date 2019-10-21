macro_rules! instrument {
    ($name:literal, $block:expr) => {
        use log::trace;
        use std::time::Instant;

        let now = Instant::now();
        trace!("[{}] start.", $name);

        $block;

        trace!("[{}] done. elapsed time {}", $name, now.elapsed().as_secs_f64());
    }
}
