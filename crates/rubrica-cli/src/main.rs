use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rubrica_core::{formats, load_pkcs12};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "rubrica",
    version,
    about = "Firma electrónica para las sedes españolas"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Sign(SignArgs),
}

#[derive(Parser)]
struct SignArgs {
    #[arg(long)]
    r#in: PathBuf,

    #[arg(long)]
    cert: PathBuf,

    #[arg(long, default_value = "")]
    pass: String,

    #[arg(long)]
    out: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Sign(args) => sign(args),
    }
}

fn sign(args: SignArgs) -> Result<()> {
    let identity = load_pkcs12(&args.cert, &args.pass)
        .with_context(|| format!("cargando {}", args.cert.display()))?;
    let input =
        std::fs::read(&args.r#in).with_context(|| format!("leyendo {}", args.r#in.display()))?;

    let signed = if is_pdf(&input) {
        formats::pades::sign(&input, &identity)?
    } else {
        formats::cades::sign(&input, &identity)?
    };

    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&args.out, signed)
        .with_context(|| format!("escribiendo {}", args.out.display()))?;

    let cn = identity
        .common_name()
        .unwrap_or_else(|| "(sin nombre)".into());
    println!("Firmado por {cn} -> {}", args.out.display());
    Ok(())
}

fn is_pdf(data: &[u8]) -> bool {
    data.starts_with(b"%PDF-")
}
