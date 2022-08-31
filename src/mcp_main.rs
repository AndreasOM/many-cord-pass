use many_cord_pass::Config;
use many_cord_pass::ManyCordPass;

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
    let mut mcp = ManyCordPass::default();
    mcp.load_config("config.yaml")?;
    dbg!(&mcp);
    /*
    match mcp.find_and_connect() {
        Ok( _ ) => {},
        Err( e ) => {
            println!("Error connecting to Stream Deck:{}\n\tUsing terminal mode!", e);
            mcp.enable_terminal_input();
        },
    };
    */

    mcp.run()?;
    while !mcp.done() {
        match mcp.update() {
            Ok( _ ) => {},
            Err( e ) => {
                println!("Error: {:?}", &e);
            },
        };
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
