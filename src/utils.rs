#[macro_export]
macro_rules! setter_copy {
    ($name: ident, $tyty: ty) => {
        #[must_use]
        pub fn $name(mut self, $name: $tyty) -> Self {
            self.$name = $name;
            self
        }
    };
}

#[macro_export]
macro_rules! setter_option_copy {
    ($name: ident, $tyty: ty) => {
        #[must_use]
        pub fn $name(mut self, $name: $tyty) -> Self {
            self.$name = Some($name);
            self
        }
    };
}

#[macro_export]
macro_rules! setter_option_clone {
    ($name: ident, $tyty: ty) => {
        #[must_use]
        pub fn $name(mut self, $name: &$tyty) -> Self {
            self.$name = Some($name.clone());
            self
        }
    };
}

#[macro_export]
macro_rules! setter_clone {
    ($name: ident, $tyty: ty) => {
        #[must_use]
        pub fn $name(mut self, $name: &$tyty) -> Self {
            self.$name = $name.clone();
            self
        }
    };
}
