use open_modular_runtime::runtime::Runtime as _;
use open_modular_runtime_production::Runtime;
use tracing::instrument;
use tracing_subscriber::{
    EnvFilter,
    fmt,
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
};

use crate::module::Module;

// =================================================================================================
// Workbench
// =================================================================================================

pub fn run() {
    configure();
    execute();
}

// -------------------------------------------------------------------------------------------------

// Configure

static FILTER_ENV_VAR: &str = "TRACE";
static FILTER_DEFAULT_LEVEL: &str = "warn";

fn configure() {
    configure_tracing();
}

fn configure_tracing() {
    let filter = EnvFilter::try_from_env(FILTER_ENV_VAR)
        .or_else(|_| EnvFilter::try_new(FILTER_DEFAULT_LEVEL))
        .expect("filter to be defined");

    let formatter = fmt::layer()
        .pretty()
        .with_level(true)
        .with_ansi(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    tracing_subscriber::registry()
        .with(formatter)
        .with(filter)
        .init();
}

// -------------------------------------------------------------------------------------------------

// Execute

#[instrument(level = "debug")]
fn execute() {
    Runtime::default().run::<Module<_>>();
}
