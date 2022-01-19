pub trait Renderer<'a, 'b> {
    fn render(&self) -> Result<(), Box<dyn std::error::Error>>;
}
