use std::cell::{Ref, RefMut};

use derive_more::{Deref, DerefMut, Display};

use crate::Resources;

#[derive(Display, Deref)]
#[deref(forward)]
pub struct Res<'a, T>(Ref<'a, T>);

#[derive(Display, Deref, DerefMut)]
#[deref(forward)]
pub struct ResMut<'a, T>(RefMut<'a, T>);

pub trait SystemParam: Sized {
    type Item<'new>;
    fn get<'b>(resources: &'b Resources) -> Result<Self::Item<'b>, String>;
}

impl<'a, T: 'static> SystemParam for Res<'a, T> {
    type Item<'new> = Res<'new, T>;
    #[inline]
    fn get<'b>(resources: &'b Resources) -> Result<Self::Item<'b>, String> {
        resources
            .get::<T>()
            .map(Res)
            .ok_or_else(|| format!("Resource Res<{}> not found!", std::any::type_name::<T>()))
    }
}

impl<'a, T: 'static> SystemParam for ResMut<'a, T> {
    type Item<'new> = ResMut<'new, T>;
    #[inline]
    fn get<'b>(resources: &'b Resources) -> Result<Self::Item<'b>, String> {
        resources
            .get_mut::<T>()
            .map(ResMut)
            .ok_or_else(|| format!("Resource ResMut<{}> not found!", std::any::type_name::<T>()))
    }
}

impl<'a, T: 'static> SystemParam for Option<Res<'a, T>> {
    type Item<'new> = Option<Res<'new, T>>;
    #[inline]
    fn get<'b>(resources: &'b Resources) -> Result<Self::Item<'b>, String> {
        Ok(resources.get::<T>().map(Res))
    }
}

impl<'a, T: 'static> SystemParam for Option<ResMut<'a, T>> {
    type Item<'new> = Option<ResMut<'new, T>>;
    #[inline]
    fn get<'b>(resources: &'b Resources) -> Result<Self::Item<'b>, String> {
        Ok(resources.get_mut::<T>().map(ResMut))
    }
}
