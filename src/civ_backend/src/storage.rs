use crate::models::User;
use std::cell::RefCell;
use std::collections::HashMap;


thread_local! {
    pub static USERS: RefCell<HashMap<String, User>> = RefCell::new(HashMap::new());
}
