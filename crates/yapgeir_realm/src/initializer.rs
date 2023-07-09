use crate::Resources;

pub trait FromResources {
    fn from(realm: &Resources) -> Self;
}

impl<T> FromResources for T
where
    T: Default,
{
    fn from(_: &Resources) -> Self {
        Self::default()
    }
}
