#![no_main]

use std::time::Duration;

use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
enum FuzzOp {
    Input(Vec<char>),
    Flush,
}

const TERM_SIZE_PX: (u32, u32) = (256, 144);

libfuzzer_sys::fuzz_target!(|ops: Vec<FuzzOp>| {
    let mut buffer = vec![0u32; (TERM_SIZE_PX.0 * TERM_SIZE_PX.1) as usize];
    let font = shitty::Unifont::new();
    let mut shitty = shitty::Terminal::new(TERM_SIZE_PX.0, TERM_SIZE_PX.1, shitty::BufMode::Single, vec![font]);
    for op in ops {
        match op {
            FuzzOp::Input(data) => {
                shitty.process(data.iter().collect::<String>().as_bytes());
            }
            FuzzOp::Flush => {
                let mut drawable = shitty::Drawable::new(
                    shitty::BGRA::from_u32_mut_slice(&mut buffer),
                    TERM_SIZE_PX.0 as usize,
                    TERM_SIZE_PX.1 as usize,
                    0,
                );
                shitty.flush(&mut drawable, Duration::ZERO);
            }
        }
    }
});
