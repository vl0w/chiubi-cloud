use indicatif::{ProgressBar, ProgressStyle};

pub fn start_spinner(msg: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(120);
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}"));
    pb.set_message(msg);
    pb
}