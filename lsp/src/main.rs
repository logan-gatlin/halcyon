mod completion;
mod diagnostics;
mod keyword_hover;
mod protocol;
mod server;
mod util;

fn main() {
    if let Err(error) = server::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
