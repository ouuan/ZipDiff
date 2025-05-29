use zip::read::ZipArchive;
use std::fs::File;

fn main() {
    let mut args = std::env::args().skip(1);
    let src = args.next().expect("no src in args");
    let dst = args.next().expect("no dst in args");
    let file = File::open(src).expect("failed to open input file");
    let mut archive = ZipArchive::new(file).expect("failed to read input ZIP");
    archive.extract(dst).expect("failed to extract");
}
