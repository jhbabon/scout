macro_rules! instrument {
    ($name:literal, $block:expr) => {
        use log::trace;
        use std::time::Instant;

        let now = Instant::now();
        trace!("[{}] start.", $name);

        $block;

        trace!(
            "[{}] done. elapsed time {}",
            $name,
            now.elapsed().as_secs_f64()
        );
    };
}

// Copy from termion:
// https://gitlab.redox-os.org/redox-os/termion/blob/master/src/macros.rs#L2
macro_rules! csi {
    ($( $l:expr ),*) => { concat!("\x1B[", $( $l ),*) };
}
