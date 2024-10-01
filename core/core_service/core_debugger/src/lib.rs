use std::env;

use formaters::core_service_formater::CoreServiceFormatter;
pub use tracing;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

pub mod formaters;

pub fn init_tracing() {
    // Получение значения переменной окружения LOG_ALL_EVENTS (true/false)
    let log_all_events = env::var("LOG_ALL_EVENTS").unwrap_or_else(|_| "false".to_string());
    let log_all_events = log_all_events.eq_ignore_ascii_case("true");

    // Создание пользовательского слоя для 'hellai_app_core_events'
    let custom_layer = fmt::layer()
        .event_format(CoreServiceFormatter)
        .with_filter(EnvFilter::new("hellai_app_core_events=trace"));

    // Настройка фильтра для default_layer на основе LOG_ALL_EVENTS
    let default_filter = if log_all_events {
        "debug"
    } else {
        "off" // Отключаем логирование, если LOG_ALL_EVENTS=false
    };

    // Создание default_layer с динамическим фильтром
    let default_layer = fmt::layer().with_filter(EnvFilter::new(default_filter));

    // Создание подписчика с обоими слоями
    let subscriber = Registry::default().with(custom_layer).with(default_layer);

    // Инициализация подписчика
    tracing::subscriber::set_global_default(subscriber)
        .expect("Не удалось установить подписчика для Tracing");
}
