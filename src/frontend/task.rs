use crate::backend::protocol::Pending;

pub fn drive<T>(slot: &mut Option<Pending<T>>, action: impl FnOnce() -> anyhow::Result<Pending<T>> ) -> Option<anyhow::Result<T>> {
    match slot {
        None => {
            if let Ok(p) = action() {
                *slot = Some(p); None
            } else {
                None
            }          
        },
        Some(p) => {
            p.try_take()
        }
    }
}