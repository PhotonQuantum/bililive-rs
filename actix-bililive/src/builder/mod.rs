mod awc;
#[cfg(test)]
mod tests;

pub type ConfigBuilder<R, U, T, S> = bililive_core::builder::Config<awc::AWCClient, R, U, T, S>;
