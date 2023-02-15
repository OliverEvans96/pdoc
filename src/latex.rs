use std::path::Path;

use anyhow::bail;

#[derive(Clone, Debug)]
pub struct Asset {
    pub data: Vec<u8>,
    pub filename: String,
}

pub fn compile_latex(
    tex: &str,
    pdf_output_path: impl AsRef<Path>,
    assets: &[Asset],
) -> anyhow::Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_dir_path = tmp_dir.path();

    let basename = "invoice";
    let tex_filename = format!("{}.tex", basename);
    let pdf_filename = format!("{}.pdf", basename);

    let tex_path = tmp_dir_path.join(tex_filename);
    let pdf_path = tmp_dir_path.join(pdf_filename);

    // Write latex file
    std::fs::write(&tex_path, tex.as_bytes())?;

    // Copy assets to compilation directory
    for asset in assets {
        let filename = tmp_dir.path().join(&asset.filename);
        std::fs::write(filename, &asset.data)?;
    }

    let mut compile_command = std::process::Command::new("pdflatex");

    compile_command.current_dir(&tmp_dir_path).arg(&tex_path);

    let exit_status = compile_command.status()?;

    if !exit_status.success() {
        bail!("Non-success exit status: {:?}", exit_status);
    }

    std::fs::copy(pdf_path, pdf_output_path)?;

    Ok(())
}
