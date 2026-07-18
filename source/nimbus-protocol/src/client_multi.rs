use crate::client::*;
use crate::command::*;

//================================================================

#[derive(Default)]
pub struct ClientMulti {
    pub client: Vec<Client>,
}

impl ClientMulti {
    pub fn update<F: FnMut(&mut Client, &CommandServer)>(
        &mut self,
        mut call: F,
    ) -> anyhow::Result<()> {
        for c in &mut self.client {
            c.update(&mut call)?;
        }

        Ok(())
    }

    pub fn send_all(&self, command: CommandClient) {
        for c in &self.client {
            c.send(command.clone());
        }
    }
}
