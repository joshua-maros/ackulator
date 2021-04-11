pub mod data;
pub mod entity;
pub mod expression;
pub mod instance;
pub mod scalar;
mod statement;
mod storage;
pub mod units;

pub mod prelude {
    pub use crate::instance::*;
    pub use crate::scalar::*;
    pub use crate::units::*;
    pub use std::cell::RefCell;
    pub use std::rc::Rc;
    pub type Rcrc<T> = Rc<RefCell<T>>;
    pub fn rcrc<T>(value: T) -> Rcrc<T> {
        Rc::new(RefCell::new(value))
    }
}
