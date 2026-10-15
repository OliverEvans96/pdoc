mod common;

use common::{
    assert_latexmk_available, assert_valid_pdf, sample_invoice, sandbox_config,
    seed_client_and_project,
};
use pdoc::{
    date::DateString,
    receipt::Receipt,
    storage::{get_beancount_dir, get_invoices_dir, get_pdfs_dir, get_receipts_dir},
};
use tempfile::TempDir;

#[test]
fn invoice_full_workflow_produces_pdf_and_beancount() -> anyhow::Result<()> {
    assert_latexmk_available();

    let temp = TempDir::new()?;
    let config = sandbox_config(&temp)?;
    let (_, project_ref) = seed_client_and_project(&config)?;

    let invoice = sample_invoice(project_ref)?;
    invoice.save(&config)?;

    let invoices_dir = get_invoices_dir(&config)?;
    let invoice_yaml = invoices_dir.join("17.yaml");
    assert!(invoice_yaml.is_file(), "expected invoice yaml at {:?}", invoice_yaml);

    let full_invoice = invoice.collect(&config)?;
    let pdf_path = full_invoice.save_pdf(&config, false)?;
    assert_valid_pdf(&pdf_path)?;

    let expected_pdf_name = "Invoice_TestUser_17.pdf";
    assert_eq!(
        pdf_path,
        get_pdfs_dir(&config)?.join(expected_pdf_name),
        "unexpected invoice PDF path"
    );

    let beancount_path = full_invoice.save_beancount(&config)?;
    let expected_beancount = get_beancount_dir(&config)?.join("Invoice_17.beancount");
    assert_eq!(beancount_path, expected_beancount);
    assert!(beancount_path.is_file());

    Ok(())
}

#[test]
fn receipt_full_workflow_produces_pdf() -> anyhow::Result<()> {
    assert_latexmk_available();

    let temp = TempDir::new()?;
    let config = sandbox_config(&temp)?;
    let (_, project_ref) = seed_client_and_project(&config)?;

    let invoice = sample_invoice(project_ref)?;
    invoice.save(&config)?;

    let receipt = Receipt {
        invoice_num: invoice.number,
        date: DateString::try_new("2023-01-14".to_owned())?,
        payment_method: "PayPal".to_owned(),
    };
    receipt.save(&config)?;

    let receipts_dir = get_receipts_dir(&config)?;
    let receipt_yaml = receipts_dir.join("17.yaml");
    assert!(receipt_yaml.is_file(), "expected receipt yaml at {:?}", receipt_yaml);

    let full_receipt = receipt.collect(&config)?;
    let pdf_path = full_receipt.save_pdf(&config, false)?;
    assert_valid_pdf(&pdf_path)?;

    let expected_pdf_name = "Receipt_TestUser_17.pdf";
    assert_eq!(
        pdf_path,
        get_pdfs_dir(&config)?.join(expected_pdf_name),
        "unexpected receipt PDF path"
    );

    Ok(())
}
