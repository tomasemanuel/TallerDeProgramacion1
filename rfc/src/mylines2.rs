use std::io::BufRead;

static CARRIAGE_RETURN: u8 = 13;
static NEW_LINE: u8 = 10;

#[derive(Debug)]
pub struct MyLines2<B> {
    buffer: B,
}

#[derive(Debug)]
pub enum MyError2 {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl<B> MyLines2<B> {
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }
}

impl<B: BufRead> Iterator for MyLines2<B> {
    type Item = Result<(Vec<u8>, i32), MyError2>;

    fn next(&mut self) -> Option<Self::Item> {
        let (line, total) = {
            let buffer = match self.buffer.fill_buf() {
                Ok(buffer) => buffer,
                Err(e) => return Some(Err(MyError2::Io(e))),
            };

            if buffer.is_empty() {
                return None;
            }

            let mut consumed = 0;
            let mut prev_byte = buffer[0];
            for byte in buffer {
                consumed += 1;
                if prev_byte == CARRIAGE_RETURN && *byte == NEW_LINE {
                    break;
                }
                prev_byte = *byte;
            }
            let vec = <&[u8]>::clone(&buffer).to_vec();

            (vec, consumed)
        };
        self.buffer.consume(total);
        Some(Ok((line, total as i32)))
    }
}
