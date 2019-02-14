extern crate auto_from;

use auto_from::auto_from;

pub enum Example {
    Int(i32),
    Vec { length: usize },
}

#[auto_from]
impl Example {
    fn ints(i: i32) -> Example {
        Example::Int(i)
    }
    fn any_vec<T>(v: Vec<T>) -> Example
    where
        T: Clone,
    {
        Example::Vec { length: v.len() }
    }
}

#[test]
fn can_transform() {
    Example::from(3);
    Example::from(vec![5u32]);
    Example::from(vec![6i32]);
    Example::from(vec![vec![6u32]]);
}
