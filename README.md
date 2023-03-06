# pdoc

`pdoc` (payment documents) is a command-line invoice / receipt generator, which stores user/client/project info as yaml files, and produces PDFs via `pdflatex` from a simple template.

## Configuration

The configuration path is `~/.config/pdoc/config.toml`.
Current options include:

* `data_dir` - directory where produced yaml and PDF files are stored
  * must be an absolute path
  * `~` will be expanded to the current user's home directory

