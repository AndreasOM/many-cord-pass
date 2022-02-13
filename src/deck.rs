//use derivative::Derivative;

//#[derive(Derivative,Default)]
//#[derivative(Debug)]
pub trait Deck {
    fn run(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    fn update(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn done(&self) -> bool {
        false
    }

    fn set_button_file(&mut self, index: u8, filename: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

impl dyn Deck {}
