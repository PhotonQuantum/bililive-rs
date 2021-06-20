#[macro_export]
macro_rules! setter_copy {
    ($name: ident, $tyty: ty) => {
        pub fn $name(mut self, $name: $tyty) -> Self {
            self.$name = $name;
            self
        }
    };
}

#[macro_export]
macro_rules! setter_option_copy {
    ($name: ident, $tyty: ty) => {
        pub fn $name(mut self, $name: $tyty) -> Self {
            self.$name = Some($name);
            self
        }
    };
}

#[macro_export]
macro_rules! setter_option_clone {
    ($name: ident, $tyty: ty) => {
        pub fn $name(mut self, $name: &$tyty) -> Self {
            self.$name = Some($name.clone());
            self
        }
    };
}

#[macro_export]
macro_rules! while_let_kill {
    ($kill: expr, $e: expr, $p: pat => $blk: block) => {
        loop {
            let fut = $e;
            let kill_fut = $kill;
            pin_mut!(fut);
            pin_mut!(kill_fut);
            match future::select(fut, kill_fut).await {
                Either::Left(($p, _)) => $blk,
                _ => break,
            }
        }
    };
}
