# pdoc

`pdoc` (payment documents) is a command-line invoice / receipt generator, which stores user/client/project info as yaml files, and produces PDFs via `pdflatex` from a simple template.

## Configuration

The configuration path is `~/.config/pdoc/config.toml`.
Current options include:

* `data_dir` - directory where produced yaml and PDF files are stored
  * must be an absolute path
  * `~` will be expanded to the current user's home directory

Project YAML files (under the data directory’s `projects/` folder) may include optional default payment terms:

```yaml
payment_terms:
  days: 14
  day_kind: business   # or calendar
```

When both fields are set, new invoices for that project skip the payment-term prompts and compute `due_date` from the invoice date. Business days count Monday–Friday only (weekends are skipped).

## Dependencies

This program requires `latexmk` to be available on the system to render PDFs (via the `texrender` crate).
