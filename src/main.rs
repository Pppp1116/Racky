use Racky::config::settings::Configuration;
use Racky::helper::init_logging;
use Racky::udev::run_udev;

fn main() -> Result<()> {
    init_logging()?;

    let config = Configuration::load_settings().as_ref();
    run_udev(config);

    Ok(())
}
