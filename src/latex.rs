use std::path::Path;
use std::{fmt, io};

use anyhow::bail;

// Inspired by https://users.rust-lang.org/t/why-doesnt-vec-u8-implement-std-fmt-write/13200/5
pub struct ToFmtWrite<T>(pub T);

impl<T> io::Write for ToFmtWrite<T>
where
    T: fmt::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let num_bytes = buf.len();

        let s = std::str::from_utf8(buf).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("UTF8 decode error: {}", err),
            )
        })?;

        let x = fmt::Write::write_str(&mut self.0, s).map_err(|err| {
            io::Error::new(io::ErrorKind::Other, format!("Error writing: {}", err))
        })?;

        Ok(num_bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}
pub struct Latex;

impl askama_escape::Escaper for Latex {
    fn write_escaped<W>(&self, fmt: W, string: &str) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        let writer = ToFmtWrite(fmt);
        texrender::tex_escape::write_escaped(writer, string).map_err(|_err| fmt::Error)
    }
}

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
