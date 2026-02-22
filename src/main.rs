use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "restore",
    about = "Extract Windows backup ZIP files and restore folder structure",
    version
)]
struct Args {
    #[arg(short, long, value_name = "PATH", help = "Path to the backup folder")]
    source: PathBuf,
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Destination path for restored files",
        required_unless_present = "analyze_only"
    )]
    dest: Option<PathBuf>,
    #[arg(short = 'a', long, help = "Analyze only, do not extract")]
    analyze_only: bool,
}

fn collect_zips(dir: &Path, zips: &mut Vec<PathBuf>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_zips(&path, zips)?;
        } else if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("zip")) {
            zips.push(path);
        }
    }
    Ok(())
}

fn cmp_natural(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let mut ai = 0usize;
    let mut bi = 0usize;

    while ai < ab.len() && bi < bb.len() {
        let a_digit = ab[ai].is_ascii_digit();
        let b_digit = bb[bi].is_ascii_digit();

        if a_digit && b_digit {
            let a_start = ai;
            while ai < ab.len() && ab[ai].is_ascii_digit() {
                ai += 1;
            }
            let b_start = bi;
            while bi < bb.len() && bb[bi].is_ascii_digit() {
                bi += 1;
            }

            let a_run = &a[a_start..ai];
            let b_run = &b[b_start..bi];
            let a_trim = a_run.trim_start_matches('0');
            let b_trim = b_run.trim_start_matches('0');

            let a_len = a_trim.len();
            let b_len = b_trim.len();
            if a_len != b_len {
                return a_len.cmp(&b_len);
            }
            let ord = a_trim.cmp(b_trim);
            if ord != Ordering::Equal {
                return ord;
            }
            let ord = a_run.len().cmp(&b_run.len());
            if ord != Ordering::Equal {
                return ord;
            }
        } else {
            let a_byte = ab[ai].to_ascii_lowercase();
            let b_byte = bb[bi].to_ascii_lowercase();
            if a_byte != b_byte {
                return a_byte.cmp(&b_byte);
            }
            ai += 1;
            bi += 1;
        }
    }

    ab.len().cmp(&bb.len())
}

fn find_zip_files(source_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut zips = Vec::new();
    collect_zips(source_dir, &mut zips)?;
    zips.sort_by(|a, b| cmp_natural(&a.to_string_lossy(), &b.to_string_lossy()));
    Ok(zips)
}

fn strip_drive_letter(path: &str) -> &str {
    let bytes = path.as_bytes();
    // Match patterns like "C/" or "C\" at start
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && (bytes[1] == b'/' || bytes[1] == b'\\') {
        &path[2..]
    } else {
        path
    }
}

fn analyze(source_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let zips = find_zip_files(source_dir)?;
    let total_size: u64 = zips.iter().filter_map(|z| fs::metadata(z).ok()).map(|m| m.len()).sum();

    println!("\n{}", "=".repeat(60));
    println!(" Windows Backup Analyzer");
    println!("{}", "=".repeat(60));
    println!(" Source directory:  {}", source_dir.display());
    println!(" ZIP files:         {}", zips.len());
    println!(" Total size:        {:.2} GB", total_size as f64 / (1024.0 * 1024.0 * 1024.0));

    // Show sample from first ZIP
    if let Some(first) = zips.first() {
        if let Ok(file) = fs::File::open(first) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                let mut extensions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for i in 0..archive.len() {
                    if let Ok(entry) = archive.by_index(i) {
                        if let Some(ext) = Path::new(entry.name()).extension() {
                            *extensions.entry(ext.to_string_lossy().to_lowercase()).or_insert(0) += 1;
                        }
                    }
                }
                println!("\n Sample from: {}", first.file_name().unwrap_or_default().to_string_lossy());
                let mut sorted: Vec<_> = extensions.into_iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(&a.1));
                for (ext, count) in sorted.iter().take(10) {
                    println!("   .{:<11} -> {} files", ext, count);
                }
            }
        }
    }
    println!("{}\n", "=".repeat(60));
    Ok(zips)
}

fn extract(source_dir: &Path, dest_dir: &Path) -> io::Result<()> {
    let zips = find_zip_files(source_dir)?;
    if zips.is_empty() {
        eprintln!("ERROR: No ZIP files found!");
        return Ok(());
    }

    fs::create_dir_all(dest_dir)?;

    let total = zips.len();
    let mut files_extracted: usize = 0;
    let mut errors: Vec<String> = Vec::new();

    println!("\nStarting extraction of {} ZIP files...", total);
    println!("Destination: {}\n", dest_dir.display());

    for (i, zip_path) in zips.iter().enumerate() {
        let zip_name = zip_path.file_name().unwrap_or_default().to_string_lossy();
        print!("[{}/{}] {}... ", i + 1, total, zip_name);
        io::stdout().flush().ok();

        match fs::File::open(zip_path) {
            Ok(file) => match zip::ZipArchive::new(file) {
                Ok(mut archive) => {
                    let mut count = 0usize;
                    for j in 0..archive.len() {
                        match archive.by_index(j) {
                            Ok(mut entry) => {
                                if entry.is_dir() {
                                    continue;
                                }
                                let raw_name = entry.name().replace('\\', "/");
                                let clean = strip_drive_letter(&raw_name);
                                let target = dest_dir.join(clean);

                                if let Some(parent) = target.parent() {
                                    if let Err(e) = fs::create_dir_all(parent) {
                                        errors.push(format!("{}: mkdir {}: {}", zip_name, parent.display(), e));
                                        continue;
                                    }
                                }

                                match fs::File::create(&target) {
                                    Ok(mut outfile) => {
                                        if let Err(e) = io::copy(&mut entry, &mut outfile) {
                                            errors.push(format!("{}: write {}: {}", zip_name, clean, e));
                                        } else {
                                            count += 1;
                                        }
                                    }
                                    Err(e) => {
                                        errors.push(format!("{}: create {}: {}", zip_name, clean, e));
                                    }
                                }
                            }
                            Err(e) => {
                                errors.push(format!("{}: entry {}: {}", zip_name, j, e));
                            }
                        }
                    }
                    files_extracted += count;
                    println!("{} files", count);
                }
                Err(e) => {
                    println!("ERROR: {}", e);
                    errors.push(format!("{}: {}", zip_name, e));
                }
            },
            Err(e) => {
                println!("ERROR: {}", e);
                errors.push(format!("{}: {}", zip_name, e));
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!(" Extraction completed!");
    println!(" Files extracted:   {}", files_extracted);
    println!(" Errors:            {}", errors.len());
    println!(" Destination:       {}", dest_dir.display());
    println!("{}", "=".repeat(60));

    if !errors.is_empty() {
        println!("\nError details:");
        for err in errors.iter().take(20) {
            println!("  {}", err);
        }
        if errors.len() > 20 {
            println!("  ... and {} more errors", errors.len() - 20);
        }
    }
    Ok(())
}


fn main() {
    let args = Args::parse();

    let source_path = args.source.as_path();
    if !source_path.is_dir() {
        eprintln!("ERROR: Directory not found: {}", source_path.display());
        std::process::exit(1);
    }

    match analyze(source_path) {
        Ok(zips) => {
            if args.analyze_only || zips.is_empty() {
                return;
            }
        }
        Err(e) => {
            eprintln!("Error during analysis: {}", e);
            std::process::exit(1);
        }
    }

    let dest = args.dest.expect("Destination path is required");

    println!("\n  Source: {}", source_path.display());
    println!("  Dest:   {}", dest.display());
    print!("\nProceed? (y/n): ");
    io::stdout().flush().ok();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();

    if confirm.trim().to_lowercase().starts_with('j') || confirm.trim().to_lowercase().starts_with('y') {
        if let Err(e) = extract(source_path, dest.as_path()) {
            eprintln!("Error during extraction: {}", e);
            std::process::exit(1);
        }
    } else {
        println!("Cancelled.");
    }
}