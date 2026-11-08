use Racky::helper::init_logging;
use Racky::udev::run_udev;

fn main() -> Result<()> {
    init_logging()?;

    run_udev();

    Ok(())
}
