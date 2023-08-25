fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    build_wasm::run(args.iter().map(|v| v.as_str()));
}
