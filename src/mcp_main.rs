#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pid = 0x0063;
    let vid = 0x0fd9;
    let serial = None;

    let mut deck = match streamdeck::StreamDeck::connect(vid, pid, serial) {
        Ok(d) => d,
        Err(e) => {
            println!("Error connecting to streamdeck: {:?}", e);
            return Err(anyhow::anyhow!("Error"));
        }
    };

    let key = 0;
    let file = "data/about.png";
    let opts = streamdeck::images::ImageOptions::default();

    deck.set_button_file(key, &file, &opts)?;

    Ok(())
}
