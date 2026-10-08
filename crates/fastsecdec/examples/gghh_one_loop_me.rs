//! Native exporter for the complete top-loop gg -> hh helicity reproduction.
#[path = "gghh_double_box/color.rs"]
mod color;
#[path = "gghh_double_box/dot.rs"]
mod dot;
#[path = "gghh_double_box/export.rs"]
mod export;
#[path = "gghh_double_box/point.rs"]
mod point;
#[path = "../../../example/gg_hh_one_loop_ME/fastsecdec/exporter.rs"]
mod reproduction;
#[path = "../../../example/gg_hh_one_loop_ME/fastsecdec/summarize.rs"]
mod summarize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("summarize") {
        summarize::run()
    } else {
        reproduction::run()
    }
}
