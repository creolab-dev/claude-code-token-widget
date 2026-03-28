use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("File watch error: {0}")]
    Watch(#[from] notify_debouncer_mini::notify::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Store error: {0}")]
    Store(String),

    #[error("Window error: {0}")]
    Window(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_json_parse_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = AppError::JsonParse(json_err);
        let display = format!("{err}");
        assert!(display.starts_with("JSON parse error:"), "got: {display}");
    }

    #[test]
    fn display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AppError::Io(io_err);
        let display = format!("{err}");
        assert!(display.starts_with("IO error:"), "got: {display}");
        assert!(display.contains("file not found"));
    }

    #[test]
    fn display_lock_error() {
        let err = AppError::Lock("mutex poisoned".to_string());
        assert_eq!(format!("{err}"), "Lock error: mutex poisoned");
    }

    #[test]
    fn display_store_error() {
        let err = AppError::Store("store failed".to_string());
        assert_eq!(format!("{err}"), "Store error: store failed");
    }

    #[test]
    fn display_window_error() {
        let err = AppError::Window("window not found".to_string());
        assert_eq!(format!("{err}"), "Window error: window not found");
    }

    #[test]
    fn serialize_produces_json_string_not_object() {
        let err = AppError::Lock("test error".to_string());
        let json = serde_json::to_value(&err).expect("serialize");
        assert!(json.is_string(), "should be a JSON string, got: {json}");
        assert_eq!(json.as_str().expect("str"), "Lock error: test error");
    }

    #[test]
    fn serialize_io_error_as_string() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = AppError::Io(io_err);
        let json = serde_json::to_value(&err).expect("serialize");
        assert!(json.is_string());
        let s = json.as_str().expect("str");
        assert!(s.starts_with("IO error:"));
        assert!(s.contains("access denied"));
    }

    #[test]
    fn serialize_json_parse_error_as_string() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        let err = AppError::JsonParse(json_err);
        let json = serde_json::to_value(&err).expect("serialize");
        assert!(json.is_string());
        let s = json.as_str().expect("str");
        assert!(s.starts_with("JSON parse error:"));
    }

    #[test]
    fn from_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    fn from_serde_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("!!!").unwrap_err();
        let app_err: AppError = json_err.into();
        assert!(matches!(app_err, AppError::JsonParse(_)));
    }
}
