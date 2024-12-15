use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeWebConsoleWriter};
use wasm_bindgen::prelude::*;

pub fn init_logger() {
    static LOGGER_INITIALIZED: std::sync::Once = std::sync::Once::new();

    LOGGER_INITIALIZED.call_once(|| {
        set_stack_trace_limit(30);

        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false) // Only partially supported across JavaScript runtimes
            .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
            .with_level(false)
            .with_target(false)
            .with_writer(MakeWebConsoleWriter::new().with_pretty_level()); // write events to the console

        let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(perf_layer)
            .init();

        std::panic::set_hook(Box::new(tracing_panic::panic_hook));
    });
}

#[wasm_bindgen(
    inline_js = "export function set_stack_trace_limit(limit) { Error.stackTraceLimit = limit; }"
)]
extern "C" {
    fn set_stack_trace_limit(limit: u32);
}
