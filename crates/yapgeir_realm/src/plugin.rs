use crate::Realm;

pub trait Plugin {
    fn register(self, realm: &mut Realm);
}

impl<F: FnOnce(&mut Realm)> Plugin for F {
    fn register(self, realm: &mut Realm) {
        self(realm)
    }
}
