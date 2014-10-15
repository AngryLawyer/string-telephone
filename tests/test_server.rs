#![feature(macro_rules)]
extern crate string_telephone;

mod test_shared;

/**
 * Test we can start listening
 */
#[test]
fn create_server() {
    unimplemented!()
}

/**
 * Test we get empty polls when we have no clients
 */
#[test]
fn empty_poll() {
    unimplemented!()
}

/**
 * Test we reject bad connection attempts
 */
#[test]
fn bad_client_attempt() {
    unimplemented!()
}

/**
 * Test we can take multiple clients
 */
#[test]
fn multiple_clients() {
    unimplemented!()
}

/**
 * Test we can cull old clients
 */
#[test]
fn cull() {
    unimplemented!()
}

/**
 * Test we can send to one
 */
#[test]
fn send_to_one() {
    unimplemented!()
}

/**
 * Test we can't send to folks who aren't connected
 */
#[test]
fn send_to_disconnected() {
    unimplemented!()
}

/**
 * Test we can send to multiple folks
 */
#[test]
fn send_to_many() {
    unimplemented!()
}

/**
 * Test we can send to everyone connected
 */
#[test]
fn send_to_all() {
    unimplemented!()
}
