//! Lists a directory or extracts a file from a game archive (read-only):
//!
//! ```text
//! cargo run -p hashfs --example extract -- def.scs def            # list the def directory
//! cargo run -p hashfs --example extract -- def.scs def/city.sii out.sii
//! ```

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = (|| -> hashfs::Result<()> {
        match args.as_slice() {
            [archive, path] => {
                let mut archive = hashfs::Archive::open(archive)?;
                match archive.list(path)? {
                    Some(listing) => {
                        for directory in listing.directories {
                            println!("{directory}/");
                        }
                        for file in listing.files {
                            println!("{file}");
                        }
                    }
                    None => println!("{path}: not a directory in this archive"),
                }
            }
            [archive, path, out] => {
                let mut archive = hashfs::Archive::open(archive)?;
                match archive.read(path)? {
                    Some(bytes) => {
                        std::fs::write(out, &bytes)?;
                        println!("{out}: {} bytes", bytes.len());
                    }
                    None => println!("{path}: not a file in this archive"),
                }
            }
            _ => println!("usage: extract <archive.scs> <dir> | <archive.scs> <file> <out>"),
        }
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
