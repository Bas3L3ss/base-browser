pub trait Connection {
    fn write(&mut self, bytes: &[u8]) -> anyhow::Result<()>;

    fn read(&mut self) -> anyhow::Result<Vec<u8>>;
}
