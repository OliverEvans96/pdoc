use std::path::Path;
use std::{fmt, io};

use anyhow::Context;
use texrender::TexRender;

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

        fmt::Write::write_str(&mut self.0, s).map_err(|err| {
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
    let mut renderer = TexRender::from_bytes(tex.as_bytes().to_vec());

    for asset in assets {
        renderer
            .add_asset_from_bytes(&asset.filename, &asset.data)
            .context("adding LaTeX asset to renderer")?;
    }

    println!("Generating PDF...");

    let pdf_data = renderer.render().context("rendering LaTeX to PDF")?;

    std::fs::write(pdf_output_path, pdf_data).context("writing rendered PDF to file")?;

    Ok(())
}
