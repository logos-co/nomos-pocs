pub trait Read {
    fn read() -> Self;
}

impl<T> Read for T
where
    T: serde::de::DeserializeOwned,
{
    fn read() -> Self {
        risc0_zkvm::guest::env::read()
    }
}

#[cfg(not(target_os = "zkvm"))]
pub trait Write {
    fn write(&self, env: &mut risc0_zkvm::ExecutorEnvBuilder);
}

#[cfg(target_os = "zkvm")]
pub trait Write {
    fn write();
}

#[cfg(not(target_os = "zkvm"))]
impl<T> Write for T
where
    T: serde::Serialize,
{
    fn write(&self, env: &mut risc0_zkvm::ExecutorEnvBuilder) {
        env.write(self).unwrap();
    }
}

#[cfg(target_os = "zkvm")]
impl<T> Write for T
where
    T: serde::Serialize,
{
    fn write(&self) {}
}
