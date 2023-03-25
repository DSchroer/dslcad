use dslcad_api::protocol::Part;
use std::io::Write;

pub fn export_txt(text: &str, mut writer: impl Write) -> std::io::Result<()> {
    writer.write(text.as_bytes())?;
    Ok(())
}
