#![no_main]
use libfuzzer_sys::fuzz_target;
use pukaar_logic::event::decode;
use pukaar_logic::store::Store;
use pukaar_logic::sync::Wire;

fuzz_target!(|data: &[u8]| {
    let _ = Wire::decode(data);
    if let Ok(e) = decode(data) {
        let mut s = Store::new(e.u.site);
        let _ = s.insert(e);
    }
});
