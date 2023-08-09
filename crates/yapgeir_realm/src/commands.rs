use crate::Resources;

#[derive(Default)]
pub struct Commands {
    commands: Vec<Box<dyn FnOnce(&mut Resources)>>,
}

impl Commands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_resource<R: 'static>(&mut self, resource: R) {
        self.add(move |resources| {
            resources.insert(resource);
        })
    }

    pub fn add<F: FnOnce(&mut Resources) + 'static>(&mut self, f: F) {
        self.commands.push(Box::new(f));
    }

    pub(crate) fn run(resources: &mut Resources) {
        while let Some(command) = resources
            .get_mut::<Self>()
            .and_then(|mut c| c.commands.pop())
        {
            command(resources)
        }
    }
}
