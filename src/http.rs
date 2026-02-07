use std::{collections::HashSet, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Context;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    client::Client,
    config::Config,
    id::Id,
    invoice::{FullInvoice, Invoice, LineItem},
    project::Project,
    receipt::{FullReceipt, Receipt},
    storage::{
        find_client, find_invoice, find_project, get_clients_dir, get_invoices_dir,
        get_projects_dir, get_receipts_dir,
    },
};

#[derive(Clone)]
struct AppState {
    config: Arc<RwLock<Config>>,
}

impl AppState {
    fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    async fn read_config(&self) -> Config {
        self.config.read().await.clone()
    }

    async fn write_config(&self, config: Config) -> Result<Config, AppError> {
        config.save()?;
        let mut guard = self.config.write().await;
        *guard = config.clone();
        Ok(config)
    }
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl AppError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let payload = ErrorResponse {
            error: self.message,
        };
        (self.status, Json(payload)).into_response()
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct FileResponse {
    path: String,
}

#[derive(Deserialize)]
struct InvoiceCreateRequest {
    number: Option<u32>,
    project_ref: Id,
    date: crate::date::DateString,
    due_date: crate::date::DateString,
    #[serde(default)]
    items: Vec<LineItem>,
    #[serde(default)]
    conditions: Option<String>,
}

#[derive(Deserialize)]
struct ReceiptCreateRequest {
    invoice_num: u32,
    date: crate::date::DateString,
    payment_method: String,
}

pub async fn serve(config: Config, host: String, port: u16) -> anyhow::Result<()> {
    let state = AppState::new(config);
    let app = Router::new()
        .route("/health", get(health))
        .route("/config", get(get_config).put(put_config))
        .route("/me", get(get_me).put(put_me))
        .route("/clients", get(list_clients).post(create_client))
        .route("/clients/:id", get(get_client).put(put_client))
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/:id", get(get_project).put(put_project))
        .route("/invoices", get(list_invoices).post(create_invoice))
        .route("/invoices/next-number", get(get_next_invoice_number))
        .route("/invoices/:number", get(get_invoice).put(put_invoice))
        .route("/invoices/:number/full", get(get_invoice_full))
        .route("/invoices/:number/tex", get(get_invoice_tex))
        .route(
            "/invoices/:number/pdf",
            get(get_invoice_pdf).post(post_invoice_pdf),
        )
        .route(
            "/invoices/:number/beancount",
            get(get_invoice_beancount).post(post_invoice_beancount),
        )
        .route("/receipts", get(list_receipts).post(create_receipt))
        .route("/receipts/unpaid", get(list_unpaid_receipts))
        .route("/receipts/:number", get(get_receipt).put(put_receipt))
        .route("/receipts/:number/full", get(get_receipt_full))
        .route("/receipts/:number/tex", get(get_receipt_tex))
        .route(
            "/receipts/:number/pdf",
            get(get_receipt_pdf).post(post_receipt_pdf),
        )
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .context("parsing listen address")?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("binding HTTP listener")?;
    axum::serve(listener, app)
        .await
        .context("serving HTTP")?;

    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn get_config(State(state): State<AppState>) -> Result<Json<Config>, AppError> {
    Ok(Json(state.read_config().await))
}

async fn put_config(
    State(state): State<AppState>,
    Json(config): Json<Config>,
) -> Result<Json<Config>, AppError> {
    if let Some(data_dir) = config.storage.data_dir.as_ref() {
        if !data_dir.is_absolute() {
            return Err(AppError::bad_request(
                "storage.data_dir must be an absolute path",
            ));
        }
    }
    let config = state.write_config(config).await?;
    Ok(Json(config))
}

async fn get_me(State(state): State<AppState>) -> Result<Json<crate::me::Me>, AppError> {
    let config = state.read_config().await;
    Ok(Json(config.me))
}

async fn put_me(
    State(state): State<AppState>,
    Json(me): Json<crate::me::Me>,
) -> Result<Json<crate::me::Me>, AppError> {
    let mut config = state.read_config().await;
    config.me = me;
    let config = state.write_config(config).await?;
    Ok(Json(config.me))
}

async fn list_clients(State(state): State<AppState>) -> Result<Json<Vec<Id>>, AppError> {
    let config = state.read_config().await;
    let clients = Client::list(&config)?;
    Ok(Json(clients))
}

async fn get_client(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Client>, AppError> {
    let config = state.read_config().await;
    let client_id = Id::new(id);
    let path = client_path(&config, &client_id)?;
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "client {:?} not found",
            client_id
        )));
    }
    let client = Client::load_from_path(path)?;
    Ok(Json(client))
}

async fn create_client(
    State(state): State<AppState>,
    Json(client): Json<Client>,
) -> Result<(StatusCode, Json<Client>), AppError> {
    let config = state.read_config().await;
    let path = client_path(&config, &client.name)?;
    if path.exists() {
        return Err(AppError::conflict("client already exists"));
    }
    client.save(&config)?;
    Ok((StatusCode::CREATED, Json(client)))
}

async fn put_client(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(client): Json<Client>,
) -> Result<Json<Client>, AppError> {
    let client_id = Id::new(id);
    if client.name != client_id {
        return Err(AppError::bad_request(
            "client name does not match path parameter",
        ));
    }
    let config = state.read_config().await;
    client.save(&config)?;
    Ok(Json(client))
}

async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<Id>>, AppError> {
    let config = state.read_config().await;
    let projects = Project::list(&config)?;
    Ok(Json(projects))
}

async fn get_project(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Project>, AppError> {
    let config = state.read_config().await;
    let project_id = Id::new(id);
    let path = project_path(&config, &project_id)?;
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "project {:?} not found",
            project_id
        )));
    }
    let project = Project::load_from_path(path)?;
    Ok(Json(project))
}

async fn create_project(
    State(state): State<AppState>,
    Json(project): Json<Project>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    let config = state.read_config().await;
    let path = project_path(&config, &project.name)?;
    if path.exists() {
        return Err(AppError::conflict("project already exists"));
    }
    if find_client(&project.client_ref, &config).is_err() {
        return Err(AppError::unprocessable("client_ref does not exist"));
    }
    project.save(&config)?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn put_project(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(project): Json<Project>,
) -> Result<Json<Project>, AppError> {
    let project_id = Id::new(id);
    if project.name != project_id {
        return Err(AppError::bad_request(
            "project name does not match path parameter",
        ));
    }
    let config = state.read_config().await;
    if find_client(&project.client_ref, &config).is_err() {
        return Err(AppError::unprocessable("client_ref does not exist"));
    }
    project.save(&config)?;
    Ok(Json(project))
}

async fn list_invoices(State(state): State<AppState>) -> Result<Json<Vec<u32>>, AppError> {
    let config = state.read_config().await;
    let invoices = Invoice::list(&config)?;
    Ok(Json(invoices))
}

async fn get_next_invoice_number(
    State(state): State<AppState>,
) -> Result<Json<u32>, AppError> {
    let config = state.read_config().await;
    let next = Invoice::get_next_number(&config)?;
    Ok(Json(next))
}

async fn get_invoice(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<Invoice>, AppError> {
    let config = state.read_config().await;
    let path = invoice_path(&config, number)?;
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "invoice {} not found",
            number
        )));
    }
    let invoice = Invoice::load_from_path(path)?;
    Ok(Json(invoice))
}

async fn create_invoice(
    State(state): State<AppState>,
    Json(request): Json<InvoiceCreateRequest>,
) -> Result<(StatusCode, Json<Invoice>), AppError> {
    let config = state.read_config().await;
    let number = match request.number {
        Some(number) => number,
        None => Invoice::get_next_number(&config)?,
    };
    if Invoice::exists(number, &config)? {
        return Err(AppError::conflict("invoice already exists"));
    }
    if find_project(&request.project_ref, &config).is_err() {
        return Err(AppError::unprocessable("project_ref does not exist"));
    }
    let invoice = Invoice {
        number,
        project_ref: request.project_ref,
        date: request.date,
        due_date: request.due_date,
        items: request.items,
        conditions: request.conditions,
    };
    invoice.save(&config)?;
    Ok((StatusCode::CREATED, Json(invoice)))
}

async fn put_invoice(
    Path(number): Path<u32>,
    State(state): State<AppState>,
    Json(invoice): Json<Invoice>,
) -> Result<Json<Invoice>, AppError> {
    if invoice.number != number {
        return Err(AppError::bad_request(
            "invoice number does not match path parameter",
        ));
    }
    let config = state.read_config().await;
    if find_project(&invoice.project_ref, &config).is_err() {
        return Err(AppError::unprocessable("project_ref does not exist"));
    }
    invoice.save(&config)?;
    Ok(Json(invoice))
}

async fn get_invoice_full(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<FullInvoice>, AppError> {
    let config = state.read_config().await;
    let invoice = load_invoice(number, &config)?;
    let full = invoice.collect(&config)?;
    Ok(Json(full))
}

async fn get_invoice_tex(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let config = state.read_config().await;
    let invoice = load_invoice(number, &config)?;
    let full = invoice.collect(&config)?;
    let rendered = full.render().context("rendering invoice template")?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        rendered,
    ))
}

async fn get_invoice_beancount(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let config = state.read_config().await;
    let invoice = load_invoice(number, &config)?;
    let full = invoice.collect(&config)?;
    let rendered = run_blocking(move || full.write_beancount_to_string()).await?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        rendered,
    ))
}

async fn post_invoice_beancount(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<FileResponse>, AppError> {
    let config = state.read_config().await;
    let invoice = load_invoice(number, &config)?;
    let full = invoice.collect(&config)?;
    let config_clone = config.clone();
    let path = run_blocking(move || full.save_beancount(&config_clone)).await?;
    Ok(Json(FileResponse {
        path: path.display().to_string(),
    }))
}

async fn get_invoice_pdf(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let config = state.read_config().await;
    let invoice = load_invoice(number, &config)?;
    let full = invoice.collect(&config)?;
    let config_clone = config.clone();
    let path = run_blocking(move || full.save_pdf(&config_clone, false)).await?;
    let bytes = run_blocking(move || {
        std::fs::read(&path).context("reading invoice PDF bytes")
    })
    .await?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/pdf")],
        bytes,
    )
        .into_response())
}

async fn post_invoice_pdf(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<FileResponse>, AppError> {
    let config = state.read_config().await;
    let invoice = load_invoice(number, &config)?;
    let full = invoice.collect(&config)?;
    let config_clone = config.clone();
    let path = run_blocking(move || full.save_pdf(&config_clone, false)).await?;
    Ok(Json(FileResponse {
        path: path.display().to_string(),
    }))
}

async fn list_receipts(State(state): State<AppState>) -> Result<Json<Vec<u32>>, AppError> {
    let config = state.read_config().await;
    let receipts = Receipt::list(&config)?;
    Ok(Json(receipts))
}

async fn list_unpaid_receipts(
    State(state): State<AppState>,
) -> Result<Json<Vec<u32>>, AppError> {
    let config = state.read_config().await;
    let invoice_nums: HashSet<u32> = Invoice::list(&config)?.into_iter().collect();
    let receipt_nums: HashSet<u32> = Receipt::list(&config)?.into_iter().collect();
    let mut unpaid: Vec<u32> = invoice_nums.difference(&receipt_nums).copied().collect();
    unpaid.sort_unstable();
    Ok(Json(unpaid))
}

async fn get_receipt(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<Receipt>, AppError> {
    let config = state.read_config().await;
    let path = receipt_path(&config, number)?;
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "receipt {} not found",
            number
        )));
    }
    let receipt = Receipt::load_from_path(path)?;
    Ok(Json(receipt))
}

async fn create_receipt(
    State(state): State<AppState>,
    Json(request): Json<ReceiptCreateRequest>,
) -> Result<(StatusCode, Json<Receipt>), AppError> {
    let config = state.read_config().await;
    let path = receipt_path(&config, request.invoice_num)?;
    if path.exists() {
        return Err(AppError::conflict("receipt already exists"));
    }
    if find_invoice(request.invoice_num, &config).is_err() {
        return Err(AppError::unprocessable("invoice_num does not exist"));
    }
    let receipt = Receipt {
        invoice_num: request.invoice_num,
        date: request.date,
        payment_method: request.payment_method,
    };
    receipt.save(&config)?;
    Ok((StatusCode::CREATED, Json(receipt)))
}

async fn put_receipt(
    Path(number): Path<u32>,
    State(state): State<AppState>,
    Json(receipt): Json<Receipt>,
) -> Result<Json<Receipt>, AppError> {
    if receipt.invoice_num != number {
        return Err(AppError::bad_request(
            "invoice_num does not match path parameter",
        ));
    }
    let config = state.read_config().await;
    if find_invoice(receipt.invoice_num, &config).is_err() {
        return Err(AppError::unprocessable("invoice_num does not exist"));
    }
    receipt.save(&config)?;
    Ok(Json(receipt))
}

async fn get_receipt_full(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<FullReceipt>, AppError> {
    let config = state.read_config().await;
    let receipt = load_receipt(number, &config)?;
    let full = receipt.collect(&config)?;
    Ok(Json(full))
}

async fn get_receipt_tex(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let config = state.read_config().await;
    let receipt = load_receipt(number, &config)?;
    let full = receipt.collect(&config)?;
    let rendered = full.render().context("rendering receipt template")?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        rendered,
    ))
}

async fn get_receipt_pdf(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let config = state.read_config().await;
    let receipt = load_receipt(number, &config)?;
    let full = receipt.collect(&config)?;
    let config_clone = config.clone();
    let path = run_blocking(move || full.save_pdf(&config_clone, false)).await?;
    let bytes = run_blocking(move || {
        std::fs::read(&path).context("reading receipt PDF bytes")
    })
    .await?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/pdf")],
        bytes,
    )
        .into_response())
}

async fn post_receipt_pdf(
    Path(number): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<FileResponse>, AppError> {
    let config = state.read_config().await;
    let receipt = load_receipt(number, &config)?;
    let full = receipt.collect(&config)?;
    let config_clone = config.clone();
    let path = run_blocking(move || full.save_pdf(&config_clone, false)).await?;
    Ok(Json(FileResponse {
        path: path.display().to_string(),
    }))
}

async fn run_blocking<T, F>(func: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(func)
        .await
        .map_err(|err| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .map_err(AppError::from)
}

fn client_path(config: &Config, id: &Id) -> anyhow::Result<PathBuf> {
    let dir = get_clients_dir(config).context("getting clients directory")?;
    Ok(dir.join(id.to_filename()))
}

fn project_path(config: &Config, id: &Id) -> anyhow::Result<PathBuf> {
    let dir = get_projects_dir(config).context("getting projects directory")?;
    Ok(dir.join(id.to_filename()))
}

fn invoice_path(config: &Config, number: u32) -> anyhow::Result<PathBuf> {
    let dir = get_invoices_dir(config).context("getting invoices directory")?;
    Ok(dir.join(format!("{}.yaml", number)))
}

fn receipt_path(config: &Config, number: u32) -> anyhow::Result<PathBuf> {
    let dir = get_receipts_dir(config).context("getting receipts directory")?;
    Ok(dir.join(format!("{}.yaml", number)))
}

fn load_invoice(number: u32, config: &Config) -> Result<Invoice, AppError> {
    let path = invoice_path(config, number)?;
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "invoice {} not found",
            number
        )));
    }
    Ok(Invoice::load_from_path(path)?)
}

fn load_receipt(number: u32, config: &Config) -> Result<Receipt, AppError> {
    let path = receipt_path(config, number)?;
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "receipt {} not found",
            number
        )));
    }
    Ok(Receipt::load_from_path(path)?)
}
