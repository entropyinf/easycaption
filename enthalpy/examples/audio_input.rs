use cpal::traits::DeviceTrait;
use enthalpy::Res;
use enthalpy::audio::input::AudioInput;

fn main() -> Res<()> {
    for host_name in AudioInput::host_names() {
        let host  = AudioInput::host_of_name(&host_name)?;
        let devices = AudioInput::devices_of_host(&host)?;
        for d in devices {
            println!("host name: {}, device name {}", host.id().name(), d.name()?);
        }
    }
    Ok(())
}
