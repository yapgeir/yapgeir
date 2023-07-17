mod initializer;
mod plugin;
mod resources;
mod systems;

pub use initializer::*;
pub use plugin::*;
pub use resources::*;
pub use systems::*;

pub use yapgeir_realm_macro::*;

pub struct Realm {
    resources: Resources,
    systems: SystemRunner,
}

impl Default for Realm {
    fn default() -> Self {
        let systems = Default::default();
        let mut resources: Resources = Default::default();
        resources.insert(Exit::default());

        Self { resources, systems }
    }
}

impl Realm {
    #[inline]
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.register(self);
        self
    }

    #[inline]
    pub fn add_system<I, S: System<()> + 'static>(
        &mut self,
        system: impl IntoSystem<I, (), System = S>,
    ) -> &mut Self {
        self.systems.push(system);
        self
    }

    pub fn run_system<I, S: System<()> + 'static>(
        &mut self,
        system: impl IntoSystem<I, (), System = S>,
    ) -> &mut Self {
        system.system().run(&mut self.resources);
        self
    }

    #[inline]
    pub fn add_resource<T: 'static>(&mut self, resource: T) -> &mut Self {
        self.resources.insert(resource);
        self
    }

    #[inline]
    pub fn initialize_resource<T: FromResources + 'static>(&mut self) -> &mut Self {
        self.resources.insert(T::from(&self.resources));
        self
    }

    #[inline]
    pub fn initialize_resource_with<T: 'static, I, S: System<T> + 'static>(
        &mut self,
        provider: impl IntoSystem<I, T, System = S>,
    ) -> &mut Self {
        let new = provider.system().run(&mut self.resources);
        self.resources.insert(new);
        self
    }

    pub fn run(&mut self) {
        loop {
            if !self.systems.run(&mut self.resources) {
                break;
            }
        }
    }
}
