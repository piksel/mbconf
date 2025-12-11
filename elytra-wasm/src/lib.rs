use std::io::Write;

#[macro_export]
macro_rules! elytra_wasm {
    ( $e:expr ) => {

        thread_local! {
            static ELYTRA_WASM_OUT: std::cell::Cell<[u64; 8]> = std::cell::Cell::new([0; 8]);
        }

        #[allow(unused)]
        #[unsafe(no_mangle)]
        pub extern "C" fn send(a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64, h: u64) -> u8 {
            let in_bytes = elytra_wasm::unpack64([a, b, c, d, e, f, g, h]);
            let res = match elytra_conf::command::Command::from_bytes(&in_bytes) {
                Ok(command) => $e(command),
                Err(e) => elytra_conf::command::CommandResponse::error(e)
            };
            if let Ok(res_bytes) = res.as_bytes().try_into() {
                ELYTRA_WASM_OUT.set(elytra_wasm::pack64(res_bytes));
                return 8;
            } else {
                return 0;
            }
        }

        #[allow(unused)]
        #[unsafe(no_mangle)]
        pub extern "C" fn recieve(index: usize) -> u64 {
            ELYTRA_WASM_OUT.get().get(index).copied().unwrap_or_default()
        }

    };
}


pub fn sync_await<T, F: Future<Output = T>>(fut: F) -> T {
    use std::task::Poll::*;
    use std::pin::pin;
    use std::task::{Context, Waker};

    let mut pin = pin!(fut);
    let mut ctx = Context::from_waker(Waker::noop());
    loop {
        match pin.as_mut().poll(&mut ctx) {
            Ready(res) => {
                return res;
            },
            Pending => {},
        }
    }
}

pub fn unpack64(value: [u64; 8]) -> [u8; 64] {
    let mut buf = [0u8; 64];
    let mut cursor = buf.as_mut_slice();
    for v in value {
        let _ = cursor.write(&v.to_be_bytes());
    }
    buf
}

pub fn pack64(value: [u8; 64]) -> [u64; 8]  {
    let mut buf = [0u64; 8];
    for (i, bytes) in value.chunks(8).enumerate() {
        let bytes = bytes.try_into()
            .expect("input array should cleanly be divisible by 8");
        buf[i] = u64::from_be_bytes(bytes);
    }
    buf
}