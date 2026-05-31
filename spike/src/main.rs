mod pades;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rubrica-spike", version, about)]
struct Args {
    #[arg(long)]
    pdf: PathBuf,

    #[arg(long)]
    p12: PathBuf,

    #[arg(long, default_value = "test")]
    pass: String,

    #[arg(long)]
    out: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let signer = pades::Signer::from_pkcs12(&args.p12, &args.pass)
        .with_context(|| format!("cargando el certificado {}", args.p12.display()))?;

    let input = std::fs::read(&args.pdf)
        .with_context(|| format!("leyendo el PDF {}", args.pdf.display()))?;

    let signed = pades::sign_b(&input, &signer).context("generando la firma PAdES-B")?;

    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&args.out, signed)
        .with_context(|| format!("escribiendo {}", args.out.display()))?;

    println!("{}", args.out.display());
    Ok(())
}
