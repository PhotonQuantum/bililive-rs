macro_rules! setter_option_copy {
    ($name: ident, $tyty: ty) => {
        #[must_use]
        pub fn $name(mut self, $name: $tyty) -> Self {
            self.$name = Some($name);
            self
        }
    };
}
