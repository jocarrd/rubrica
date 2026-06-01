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
    Verify(VerifyArgs),
}

#[derive(Parser)]
struct VerifyArgs {
    file: PathBuf,
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

    #[arg(long)]
    timestamp: bool,

    #[arg(long)]
    tsa: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Sign(args) => sign(args),
        Command::Verify(args) => verify(args),
    }
}

fn verify(args: VerifyArgs) -> Result<()> {
    let data =
        std::fs::read(&args.file).with_context(|| format!("leyendo {}", args.file.display()))?;
    let report = formats::pades::verify(&data)?;

    let signer = report
        .signer_common_name
        .as_deref()
        .unwrap_or("(desconocido)");
    println!("Firmante: {signer}");
    println!(
        "Integridad del documento: {}",
        if report.digest_matches {
            "correcta"
        } else {
            "ALTERADA"
        }
    );
    println!(
        "Firma criptográfica: {}",
        if report.signature_valid {
            "válida"
        } else {
            "INVÁLIDA"
        }
    );

    if report.is_valid() {
        println!("Resultado: firma válida");
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn sign(args: SignArgs) -> Result<()> {
    let identity = load_pkcs12(&args.cert, &args.pass)
        .with_context(|| format!("cargando {}", args.cert.display()))?;
    let input =
        std::fs::read(&args.r#in).with_context(|| format!("leyendo {}", args.r#in.display()))?;

    let tsa = args.tsa.as_deref();
    let signed = match (is_pdf(&input), args.timestamp) {
        (true, false) => formats::pades::sign(&input, &identity)?,
        (true, true) => formats::pades::sign_timestamped(&input, &identity, tsa)?,
        (false, false) => formats::cades::sign(&input, &identity)?,
        (false, true) => formats::cades::sign_timestamped(&input, &identity, tsa)?,
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
