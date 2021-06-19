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
