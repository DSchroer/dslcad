use crate::Output;
use std::io::Write;

pub fn export_txt(out: &Output, mut writer: impl Write) -> std::io::Result<usize> {
    writer.write(out.text().as_bytes())
}
