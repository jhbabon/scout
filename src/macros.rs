/// Create a CSI sequence
///
/// This is a copy from [termion](https://gitlab.redox-os.org/redox-os/termion/-/blob/a448f510f0b525896fed9f14a53030203d7f19ad/src/macros.rs#L2)
macro_rules! csi {
    ($( $l:expr ),*) => { concat!("\x1B[", $( $l ),*) };
}
