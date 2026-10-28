fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = scriptcast_media::run(&args) {
        eprintln!("scriptcast-media: {err}");
        std::process::exit(1);
    }
}
