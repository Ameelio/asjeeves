use std::fmt;
use std::ops::Deref;
use std::pin::Pin;

use zeroize::{Zeroize, Zeroizing};

/// NewType wrapper for sensitive values
/// - Uses `Zeroizing` to ensure the value is zero'ed out when
///   it leaves scope.
/// - Uses `Debug` to ensure its not leaked through logs and tracing.
/// - Uses `Pin` to ensure it cannot move (which can cause leakes)
pub struct Sensitive<T: Zeroize>(Pin<Box<Zeroizing<T>>>);

impl<T> Sensitive<T>
where
    T: Zeroize,
{
    pub fn new(inner: T) -> Self {
        let inner = Zeroizing::new(inner);
        let inner = Box::pin(inner);

        Self(inner)
    }
}
impl<T> fmt::Debug for Sensitive<T>
where
    T: Zeroize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sensitive").finish_non_exhaustive()
    }
}

impl<T> Deref for Sensitive<T>
where
    T: Deref + Zeroize,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ops::Deref;

    #[test]
    fn it_hides_sensitive_info() {
        let data = vec![1u8, 2u8];
        let s: Sensitive<Vec<u8>> = Sensitive::new(data);

        let dbg = format!("{:?}", s);

        assert_eq!("Sensitive { .. }", dbg);
        assert_eq!(&vec![1u8, 2u8], s.deref());
    }
}
