pub fn panic_handler(info: &::core::panic::PanicInfo) {
    println!("PANIC!");
    if let Some(location) = info.location() {
        println!("On {} line {}.", location.file(), location.line());
    }

    if let Some(message) = info.message() {
        println!("Message: {}", message);
    }

    if let Some(payload) = info.payload().downcast_ref::<&str>() {
        println!("Payload: {}", payload);
    }
}
