use std::{fs::File, sync::Mutex};

pub fn init_logging() -> std::io::Result<()> {
    if std::env::var_os("RACKY_LOG").is_some() {
        let file = File::create("racky.log")?;

        tracing_subscriber::fmt()
            .with_writer(Mutex::new(file))
            .with_ansi(false)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    Ok(())
}
