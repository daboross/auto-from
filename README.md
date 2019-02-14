auto-from
=========

Example usage of attribute proc macros. Mostly for my own reference and testing to reproduce issues
in more complicated proc macros.

Transforms this:

```rust
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
```

into this:

```rust
pub enum Example {
    Int(i32),
    Vec { length: usize },
}
impl ::std::convert::From<i32> for Example {
    fn from(i: i32) -> Example {
        Example::Int(i)
    }
}
impl<T> ::std::convert::From<Vec<T>> for Example
where
    T: Clone,
{
    fn from(v: Vec<T>) -> Example {
        Example::Vec { length: v.len() }
    }
}
```
