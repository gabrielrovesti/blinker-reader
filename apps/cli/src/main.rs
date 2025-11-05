use std::path::PathBuf;

fn print_usage() {
    eprintln!("Usage: blinker-cli scan <DIR> <DB_PATH>");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 3 || args[0].as_str() != "scan" {
        print_usage();
        std::process::exit(2);
    }

    let dir = PathBuf::from(args.remove(1));
    let db_path = PathBuf::from(args.remove(1));

    println!("Blinker CLI — scanning {:?} -> {:?}", dir, db_path);

    let db = match blinker_core_library::LibraryDatabase::new(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open DB: {}", e);
            std::process::exit(1);
        }
    };

    let scanner = blinker_core_library::LibraryScanner::new();
    let paths = vec![dir.as_path()];
    match scanner.scan_paths(&db, &paths).await {
        Ok(report) => {
            println!("Scan complete:");
            println!("  total:   {}", report.total);
            println!("  new:     {}", report.new);
            println!("  updated: {}", report.updated);
            if !report.errors.is_empty() {
                println!("  errors ({}):", report.errors.len());
                for e in report.errors.iter().take(10) { println!("    - {}", e); }
                if report.errors.len() > 10 { println!("    ..."); }
            }
        }
        Err(e) => {
            eprintln!("Scan error: {}", e);
            std::process::exit(1);
        }
    }
}

