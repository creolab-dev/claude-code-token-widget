use serde::{Deserialize, Serialize};

/// Claude Code ステータスラインから受信するトークン使用量データ
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsageData {
    #[serde(default)]
    pub context_window: ContextWindow,
    #[serde(default)]
    pub cost: CostInfo,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub rate_limits: Option<RateLimits>,
    #[serde(default)]
    pub model: Option<ModelInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextWindow {
    #[serde(default)]
    pub total_input_tokens: u64,
    #[serde(default)]
    pub total_output_tokens: u64,
    #[serde(default)]
    pub used_percentage: f64,
    #[serde(default)]
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurrentUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostInfo {
    #[serde(default)]
    pub total_cost_usd: f64,
    #[serde(default)]
    pub total_duration_ms: u64,
    #[serde(default)]
    pub total_api_duration_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimits {
    #[serde(default)]
    pub five_hour: Option<RateLimitEntry>,
    #[serde(default)]
    pub seven_day: Option<RateLimitEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimitEntry {
    #[serde(default)]
    pub used_percentage: f64,
    #[serde(default)]
    pub resets_at: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// ウィジェット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetSettings {
    #[serde(default = "default_currency")]
    pub currency: Currency,
    #[serde(default = "default_jpy_rate")]
    pub jpy_rate: f64,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub font_size: u8,
    #[serde(default)]
    pub display_items: Option<DisplayItems>,
    #[serde(default)]
    pub alert_settings: Option<AlertSettings>,
    #[serde(default)]
    pub schedule_settings: Option<ScheduleSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayItems {
    #[serde(default = "default_true")]
    pub rate_limits: bool,
    #[serde(default = "default_true")]
    pub context_window: bool,
    #[serde(default = "default_true")]
    pub token_stats: bool,
    #[serde(default = "default_true")]
    pub cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    #[default]
    Usd,
    Jpy,
}

impl Default for WidgetSettings {
    fn default() -> Self {
        Self {
            currency: Currency::Usd,
            jpy_rate: 150.0,
            opacity: 0.9,
            always_on_top: true,
            theme: "dark".to_string(),
            font_size: 13,
            display_items: None,
            alert_settings: None,
            schedule_settings: None,
        }
    }
}

fn default_currency() -> Currency {
    Currency::Usd
}
fn default_jpy_rate() -> f64 {
    150.0
}
fn default_opacity() -> f64 {
    0.9
}
fn default_true() -> bool {
    true
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_font_size() -> u8 {
    13
}

/// アラート通知設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_thresholds")]
    pub thresholds: Vec<u8>,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            thresholds: default_thresholds(),
        }
    }
}

fn default_thresholds() -> Vec<u8> {
    vec![50, 75, 90]
}

/// Wake-upスケジュール設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_wake_hour")]
    pub wake_hour: u8,
    #[serde(default)]
    pub wake_minute: u8,
    #[serde(default = "default_prompt")]
    pub prompt: String,
    #[serde(default = "default_command")]
    pub command: String,
}

impl Default for ScheduleSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_hour: 7,
            wake_minute: 0,
            prompt: "hello".to_string(),
            command: "claude".to_string(),
        }
    }
}

fn default_wake_hour() -> u8 {
    7
}
fn default_prompt() -> String {
    "hello".to_string()
}
fn default_command() -> String {
    "claude".to_string()
}

/// ヒートマップエントリ（日次データ）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapEntry {
    pub date: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub session_count: u32,
}

/// ウォッチャーステータスのイベントペイロード
#[derive(Debug, Clone, Serialize)]
pub struct WatcherStatus {
    pub active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_complete_json_with_f64_percentage() {
        let json = r#"{
            "context_window": {
                "total_input_tokens": 1000,
                "total_output_tokens": 500,
                "used_percentage": 57.99999999999999
            },
            "cost": {
                "total_cost_usd": 0.1234,
                "total_duration_ms": 5000,
                "total_api_duration_ms": 4000
            },
            "session_id": "test-session",
            "rate_limits": {
                "five_hour": {
                    "used_percentage": 25.5,
                    "resets_at": 1700000000
                },
                "seven_day": {
                    "used_percentage": 10.0,
                    "resets_at": 1700600000
                }
            },
            "model": {
                "id": "claude-opus-4-5-20250514",
                "display_name": "Claude Opus 4.5"
            }
        }"#;

        let data: TokenUsageData = serde_json::from_str(json).expect("should parse");
        assert!((data.context_window.used_percentage - 57.99999999999999).abs() < f64::EPSILON);
        assert_eq!(data.context_window.total_input_tokens, 1000);
        assert_eq!(data.context_window.total_output_tokens, 500);
        assert!((data.cost.total_cost_usd - 0.1234).abs() < f64::EPSILON);
        assert_eq!(data.cost.total_duration_ms, 5000);
        assert_eq!(data.session_id, Some("test-session".to_string()));

        let rl = data.rate_limits.expect("rate_limits should be present");
        let five_hour = rl.five_hour.expect("five_hour should be present");
        assert!((five_hour.used_percentage - 25.5).abs() < f64::EPSILON);
        assert_eq!(five_hour.resets_at, Some(1_700_000_000));

        let model = data.model.expect("model should be present");
        assert_eq!(model.id, Some("claude-opus-4-5-20250514".to_string()));
    }

    #[test]
    fn deserialize_empty_json_all_defaults() {
        let data: TokenUsageData = serde_json::from_str("{}").expect("should parse empty");
        assert_eq!(data.context_window.total_input_tokens, 0);
        assert_eq!(data.context_window.total_output_tokens, 0);
        assert!((data.context_window.used_percentage - 0.0).abs() < f64::EPSILON);
        assert!((data.cost.total_cost_usd - 0.0).abs() < f64::EPSILON);
        assert!(data.session_id.is_none());
        assert!(data.rate_limits.is_none());
        assert!(data.model.is_none());
    }

    #[test]
    fn deserialize_partial_json_only_context_window() {
        let json = r#"{
            "context_window": {
                "total_input_tokens": 42,
                "used_percentage": 12.345
            }
        }"#;

        let data: TokenUsageData = serde_json::from_str(json).expect("should parse partial");
        assert_eq!(data.context_window.total_input_tokens, 42);
        assert_eq!(data.context_window.total_output_tokens, 0);
        assert!((data.context_window.used_percentage - 12.345).abs() < f64::EPSILON);
        assert!((data.cost.total_cost_usd - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let data = TokenUsageData {
            context_window: ContextWindow {
                total_input_tokens: 100,
                total_output_tokens: 200,
                used_percentage: 33.33,
                current_usage: Some(CurrentUsage {
                    input_tokens: 50,
                    output_tokens: 60,
                    cache_creation_input_tokens: 10,
                    cache_read_input_tokens: 20,
                }),
            },
            cost: CostInfo {
                total_cost_usd: 1.5,
                total_duration_ms: 3000,
                total_api_duration_ms: 2500,
            },
            session_id: Some("session-123".to_string()),
            rate_limits: Some(RateLimits {
                five_hour: Some(RateLimitEntry {
                    used_percentage: 45.0,
                    resets_at: Some(1_700_000_000),
                }),
                seven_day: None,
            }),
            model: Some(ModelInfo {
                id: Some("test-model".to_string()),
                display_name: Some("Test Model".to_string()),
            }),
        };

        let json = serde_json::to_string(&data).expect("serialize");
        let roundtrip: TokenUsageData = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(roundtrip.context_window.total_input_tokens, 100);
        assert_eq!(roundtrip.context_window.total_output_tokens, 200);
        assert!((roundtrip.context_window.used_percentage - 33.33).abs() < f64::EPSILON);
        assert_eq!(roundtrip.session_id, Some("session-123".to_string()));
        assert!((roundtrip.cost.total_cost_usd - 1.5).abs() < f64::EPSILON);

        let cu = roundtrip
            .context_window
            .current_usage
            .expect("current_usage roundtrip");
        assert_eq!(cu.input_tokens, 50);
        assert_eq!(cu.cache_creation_input_tokens, 10);
    }

    #[test]
    fn widget_settings_default_values() {
        let settings = WidgetSettings::default();
        assert!(matches!(settings.currency, Currency::Usd));
        assert!((settings.jpy_rate - 150.0).abs() < f64::EPSILON);
        assert!((settings.opacity - 0.9).abs() < f64::EPSILON);
        assert!(settings.always_on_top);
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.font_size, 13);
        assert!(settings.display_items.is_none());
        assert!(settings.alert_settings.is_none());
        assert!(settings.schedule_settings.is_none());
    }

    #[test]
    fn widget_settings_camel_case_serialization() {
        let settings = WidgetSettings::default();
        let json_value = serde_json::to_value(&settings).expect("serialize");
        let obj = json_value.as_object().expect("should be object");

        // Verify camelCase keys
        assert!(obj.contains_key("currency"));
        assert!(obj.contains_key("jpyRate"));
        assert!(obj.contains_key("opacity"));
        assert!(obj.contains_key("alwaysOnTop"));
        assert!(obj.contains_key("theme"));
        assert!(obj.contains_key("fontSize"));
        assert!(obj.contains_key("displayItems"));
        assert!(obj.contains_key("alertSettings"));
        assert!(obj.contains_key("scheduleSettings"));

        // Verify snake_case keys are NOT present
        assert!(!obj.contains_key("jpy_rate"));
        assert!(!obj.contains_key("always_on_top"));
        assert!(!obj.contains_key("font_size"));
        assert!(!obj.contains_key("display_items"));
        assert!(!obj.contains_key("alert_settings"));
        assert!(!obj.contains_key("schedule_settings"));
    }

    #[test]
    fn alert_settings_default_values() {
        let settings = AlertSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.thresholds, vec![50, 75, 90]);
    }

    #[test]
    fn schedule_settings_default_values() {
        let settings = ScheduleSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.wake_hour, 7);
        assert_eq!(settings.wake_minute, 0);
        assert_eq!(settings.prompt, "hello");
        assert_eq!(settings.command, "claude");
    }

    #[test]
    fn display_items_camel_case_serialization() {
        let items = DisplayItems {
            rate_limits: true,
            context_window: false,
            token_stats: true,
            cost: false,
        };
        let json_value = serde_json::to_value(&items).expect("serialize");
        let obj = json_value.as_object().expect("should be object");

        assert!(obj.contains_key("rateLimits"));
        assert!(obj.contains_key("contextWindow"));
        assert!(obj.contains_key("tokenStats"));
        assert!(obj.contains_key("cost"));

        assert!(!obj.contains_key("rate_limits"));
        assert!(!obj.contains_key("context_window"));
        assert!(!obj.contains_key("token_stats"));
    }

    #[test]
    fn currency_serialization_lowercase() {
        let usd = Currency::Usd;
        let jpy = Currency::Jpy;

        let usd_json = serde_json::to_string(&usd).expect("serialize usd");
        let jpy_json = serde_json::to_string(&jpy).expect("serialize jpy");

        assert_eq!(usd_json, r#""usd""#);
        assert_eq!(jpy_json, r#""jpy""#);

        // Roundtrip
        let usd_rt: Currency = serde_json::from_str(&usd_json).expect("deserialize usd");
        let jpy_rt: Currency = serde_json::from_str(&jpy_json).expect("deserialize jpy");
        assert!(matches!(usd_rt, Currency::Usd));
        assert!(matches!(jpy_rt, Currency::Jpy));
    }

    #[test]
    fn heatmap_entry_roundtrip() {
        let entry = HeatmapEntry {
            date: "2025-01-15".to_string(),
            total_input_tokens: 50_000,
            total_output_tokens: 10_000,
            total_cost_usd: 2.5678,
            session_count: 3,
        };

        let json = serde_json::to_string(&entry).expect("serialize");
        let roundtrip: HeatmapEntry = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(roundtrip.date, "2025-01-15");
        assert_eq!(roundtrip.total_input_tokens, 50_000);
        assert_eq!(roundtrip.total_output_tokens, 10_000);
        assert!((roundtrip.total_cost_usd - 2.5678).abs() < f64::EPSILON);
        assert_eq!(roundtrip.session_count, 3);
    }
}
