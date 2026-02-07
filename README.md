# pdoc

`pdoc` (payment documents) is a command-line invoice / receipt generator, which stores user/client/project info as yaml files, and produces PDFs via `pdflatex` from a simple template.

## Configuration

The configuration path is `~/.config/pdoc/config.toml`.
Current options include:

* `data_dir` - directory where produced yaml and PDF files are stored
  * must be an absolute path
  * `~` will be expanded to the current user's home directory


## Dependencies

This program requires `latexmk` to be available on the system to render PDFs (via the `texrender` crate).

## HTTP API

Run the server with:

```
pdoc serve --host 127.0.0.1 --port 8080
```

Core endpoints:

- `GET /health`
- `GET /config`, `PUT /config`
- `GET /me`, `PUT /me`
- `GET /clients`, `POST /clients`, `GET /clients/:id`, `PUT /clients/:id`
- `GET /projects`, `POST /projects`, `GET /projects/:id`, `PUT /projects/:id`
- `GET /invoices`, `POST /invoices`, `GET /invoices/next-number`,
  `GET /invoices/:number`, `PUT /invoices/:number`
- `GET /invoices/:number/full`, `GET /invoices/:number/tex`
- `GET /invoices/:number/pdf`, `POST /invoices/:number/pdf`
- `GET /invoices/:number/beancount`, `POST /invoices/:number/beancount`
- `GET /receipts`, `POST /receipts`, `GET /receipts/unpaid`,
  `GET /receipts/:number`, `PUT /receipts/:number`
- `GET /receipts/:number/full`, `GET /receipts/:number/tex`
- `GET /receipts/:number/pdf`, `POST /receipts/:number/pdf`

Example create client request:

```
curl -X POST http://127.0.0.1:8080/clients \
  -H 'Content-Type: application/json' \
  -d '{"name":"Acme","address":{"addr1":"123 Main","addr2":null,"addr3":null,"city":"Metro","state":"NY","zip":"10101"},"contact":{"email":"ap@acme.com","phone":"(555) 111-2222"}}'
```
