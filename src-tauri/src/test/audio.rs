use cpal::traits::{DeviceTrait, HostTrait};
use cpal::host_from_id;
use std::fmt::Display;


fn main()  {
    cpal::available_hosts().iter().for_each(|id| {
        let host= host_from_id(*id).unwrap();

        host.input_devices().unwrap().for_each(|device| {
            println!("{:?}", device.name());
        });
    });


}
