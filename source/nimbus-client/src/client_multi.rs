use crate::client::*;
use nimbus_common::command::*;

//================================================================

#[derive(Default)]
pub struct ClientMulti {
    pub client: Vec<Client>,
}

impl ClientMulti {
    pub fn update<F: FnMut(CommandServer)>(&mut self, mut call: F) -> anyhow::Result<()> {
        for c in &mut self.client {
            c.update(&mut call)?;
        }

        Ok(())
    }

    pub fn send_all(&self, command: CommandClient) -> anyhow::Result<()> {
        for c in &self.client {
            c.send(command.clone())?;
        }

        Ok(())
    }
}
