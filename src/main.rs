use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: jb <source_dir> <target_dir>");
    }

    let source_dir = args[1].clone();
    let target_dir = args[2].clone();

    let joplin_files = jb::joplin_file_io::build_joplin_files(&source_dir).unwrap_or_else(|e| {
        eprintln!("Error building Joplin files: {}", e);
        std::process::exit(1);
    });

    jb::joplin_file_io::write_joplin_files(&target_dir, &joplin_files).unwrap_or_else(|e| {
        eprintln!("Error writing Joplin files: {}", e);
        std::process::exit(1);
    });

    jb::joplin_file_io::copy_resources(&source_dir, &target_dir).unwrap_or_else(|e| {
        eprintln!("Error copying resources: {}", e);
        std::process::exit(1);
    });

    println!("Done\n");
}
