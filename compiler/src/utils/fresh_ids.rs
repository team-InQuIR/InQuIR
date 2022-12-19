use once_cell::sync::Lazy;
use std::sync::Mutex;

static FRESH_VAR_ID: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));
static FRESH_ENT_ID: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));
static FRESH_LABEL_ID: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));

fn update_and_get(state: &Lazy<Mutex<u32>>) -> u32 {
    let mut state = state.lock().expect("lock failure");
    let val = *state;
    *state += 1;
    val
}

pub fn fresh_var_id() -> u32 {
    update_and_get(&FRESH_VAR_ID)
}

pub fn fresh_ent_id() -> u32 {
    update_and_get(&FRESH_ENT_ID)
}

pub fn fresh_label_id() -> u32 {
    update_and_get(&FRESH_LABEL_ID)
}
