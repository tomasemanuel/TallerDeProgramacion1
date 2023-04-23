use std::io::BufRead;
use std::str;
static CARRIAGE_RETURN: u8 = 13;
static NEW_LINE: u8 = 10;

#[derive(Debug)]
pub struct MyLines<B> {
    buffer: B,
}

#[derive(Debug)]
pub enum MyError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl<B> MyLines<B> {
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }
}

impl<B: BufRead> Iterator for MyLines<B> {
    type Item = Result<String, MyError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (line, total) = {
            let buffer = match self.buffer.fill_buf() {
                Ok(buffer) => buffer,
                Err(e) => return Some(Err(MyError::Io(e))),
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

            let line = match str::from_utf8(&buffer[..consumed]) {
                Ok(line) => line.to_string(),
                Err(e) => return Some(Err(MyError::Utf8(e))),
            };
            (line, consumed)
        };
        self.buffer.consume(total);

        Some(Ok(line))
    }
}

// #[cfg(test)]
// fn test_parser_1(){

//     let buff = "cadena de texto de prueba \r\n".to_string();
//     let mylines = MyLines::new(buff);
//     assert(mylines.next() == "cadena de texto de prueba \r\n");

// }
