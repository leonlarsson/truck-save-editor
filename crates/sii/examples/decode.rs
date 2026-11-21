//! Decodes any SII file to text: `cargo run -p sii --example decode -- <in.sii> <out.txt>`.
//! For looking inside saves while developing. Run it on copies, never on the live save folder.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, input, output] = args.as_slice() else {
        eprintln!("usage: decode <in.sii> <out.txt>");
        std::process::exit(2);
    };
    let result = std::fs::read(input)
        .map_err(|e| e.to_string())
        .and_then(|bytes| sii::read(&bytes).map_err(|e| e.to_string()))
        .and_then(|document| {
            std::fs::write(output, sii::to_text(&document)).map_err(|e| e.to_string())?;
            Ok(document.len())
        });
    match result {
        Ok(units) => println!("{output}: {units} units"),
        Err(error) => {
            eprintln!("{input}: {error}");
            std::process::exit(1);
        }
    }
}
