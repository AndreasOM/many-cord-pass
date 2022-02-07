fn find_deck() -> (u16, u16, Option<String>) {
    let hid = hidapi::HidApi::new().expect("could not connect to hidapi");
    let device = hid
        .device_list()
        //.filter(|item| item.product_id() == 0x006d)
        .filter(|item| {
            if let Some(product) = &item.product_string() {
                dbg!(&product);
                product.contains("Stream Deck")
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

fn fill(deck: &mut streamdeck::StreamDeck, delay: u64, r: u8, g: u8, b: u8) -> anyhow::Result<()> {
    let c = streamdeck::Colour { r, g, b };
    for k in 0..=14 {
        deck.set_button_rgb(k, &c);
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (vid, pid, serial) = find_deck();

    let mut deck = match streamdeck::StreamDeck::connect(vid, pid, serial) {
        Ok(d) => d,
        Err(e) => {
            println!("Error connecting to streamdeck: {:?}", e);
            return Err(anyhow::anyhow!("Error"));
        }
    };

    let version = deck.version()?;
    println!("Firmware Version: {}", &version);

    let key = 4;
    let file = "data/about.png";
    let file2 = "data/about_active.png";
    let opts = streamdeck::images::ImageOptions::default();

    println!("Setting button {} to {}", key, &file);
    deck.set_button_file(key, &file, &opts)?;

    loop {
        match deck.read_buttons(Some(std::time::Duration::from_millis(60))) {
            Ok(buttons) => {
                //dbg!(&buttons);
                if buttons[key as usize] == 1 {
                    deck.set_button_file(key, &file2, &opts)?;
                } else {
                    deck.set_button_file(key, &file, &opts)?;
                };
                if buttons[10] == 1 {
                    fill(&mut deck, 20, 128, 255, 64);
                };
                if buttons[11] == 1 {
                    fill(&mut deck, 20, 64, 128, 255);
                };
                if buttons[0] == 1 {
                    fill(&mut deck, 20, 255, 128, 64);
                    fill(&mut deck, 20, 0, 0, 0);
                    std::process::exit(0);
                };
            }
            Err(_) => {}
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
