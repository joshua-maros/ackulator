pub mod instance;
pub mod scalar;
pub mod units;

pub mod prelude {
    pub use crate::instance::*;
    pub use crate::scalar::*;
    pub use crate::units::*;
    pub use std::rc::Rc;
    pub use std::cell::RefCell;
    pub type Rcrc<T> = Rc<RefCell<T>>;
    pub fn rcrc<T>(value: T) -> Rcrc<T> {
        Rc::new(RefCell::new(value))
    }
}
