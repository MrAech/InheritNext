use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;

pub fn force_retry(id: u64, caller: &str) -> Result<(), CivError> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(caller) {
            if let Some(q) = u.retry_queue.as_mut() {
                if let Some(item) = q.iter_mut().find(|r| r.id == id) {
                    if item.terminal {
                        return Err(CivError::Other("retry_terminal".into()));
                    }
                    item.next_attempt_after = now_secs();
                    return Ok(());
                }
            }
            Err(CivError::Other("retry_not_found".into()))
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn force_all_due(caller: &str) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(caller) {
            if let Some(q) = u.retry_queue.as_mut() {
                let now = now_secs();
                for it in q.iter_mut() {
                    if !it.terminal {
                        it.next_attempt_after = now;
                    }
                }
            }
        }
    });
}
