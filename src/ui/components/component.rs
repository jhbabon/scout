use crate::common::Result;
use crate::state::State;

pub trait Component {
    fn update(&mut self, state: &State) -> Result<()>;
}
