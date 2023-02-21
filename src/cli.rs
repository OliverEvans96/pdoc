use inquire::validator::{StringValidator, Validation};

fn format_title(text: &str) -> String {
    let hbar = "=".repeat(text.len() + 4);
    let title_line = format!("= {} =", text);

    let lines = [String::new(), hbar.clone(), title_line, hbar, String::new()];
    let all_lines = lines.join("\n");

    all_lines
}

pub fn print_title(text: &str) {
    let formatted = format_title(text);
    println!("{}", formatted);
}

fn format_header(text: &str) -> String {
    let hbar = "=".repeat(text.len());

    let lines = [String::new(), text.to_owned(), hbar, String::new()];
    let all_lines = lines.join("\n");

    all_lines
}

pub fn print_header(text: &str) {
    let formatted = format_header(text);
    println!("{}", formatted);
}

#[cfg(test)]
mod tests {
    use super::{format_header, format_title};

    #[test]
    fn test_format_title() {
        let formatted = format_title("Potato");
        let expected = concat!("\n", "==========\n", "= Potato =\n", "==========\n",);

        assert_eq!(formatted, expected)
    }

    #[test]
    fn test_format_header() {
        let formatted = format_header("Potato");
        let expected = concat!("\n", "Potato\n", "======\n",);

        assert_eq!(formatted, expected)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NumberValidator;

impl NumberValidator {
    pub fn new() -> Self {
        Self {}
    }
}

impl StringValidator for NumberValidator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        let is_valid = input.chars().all(|c| c.is_numeric());
        let validation = if is_valid {
            Validation::Valid
        } else {
            let msg = inquire::validator::ErrorMessage::Custom("number required.".to_owned());
            Validation::Invalid(msg)
        };

        Ok(validation)
    }
}
