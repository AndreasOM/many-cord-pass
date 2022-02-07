
fn find_deck() -> (u16, u16, Option<String>) {
    let hid = hidapi::HidApi::new().expect("could not connect to hidapi");
    let device = hid
        .device_list()
        //.filter(|item| item.product_id() == 0x006d)
        .filter(|item|{
            if let Some( product ) = &item.product_string() {
              dbg!(&product);
                product.contains( "Stream Deck" )
            } else {
                false
            }
            //dbg!(&item); item.product_id() == 0x006d
        })
        .next()
        .expect("Could not find Streamdeck");

    println!(
        "Attempting to connect to {:?}. vid: {:?}, pid: {:?}, serial: {:?}",
        device.product_string(),
        device.vendor_id(),
        device.product_id(),
        device.serial_number(),
    );

    (
        device.vendor_id(),
        device.product_id(),
        device.serial_number().map(|str| String::from(str)),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let( vid, pid, serial ) = find_deck();

    let mut deck = match streamdeck::StreamDeck::connect(vid, pid, serial) {
        Ok(d) => d,
        Err(e) => {
            println!("Error connecting to streamdeck: {:?}", e);
            return Err(anyhow::anyhow!("Error"));
        }
    };

    let version = deck.version()?;
    println!("Firmware Version: {}", &version );

    let key = 4;
    let file = "data/about.png";
    let opts = streamdeck::images::ImageOptions::default();

    println!("Setting button {} to {}", key, &file);
    deck.set_button_file(key, &file, &opts)?;

    Ok(())
}
