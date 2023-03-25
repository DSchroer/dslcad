use std::io::Write;

pub fn export_txt(text: &str, mut writer: impl Write) -> std::io::Result<()> {
    writer.write_all(text.as_bytes())?;
    Ok(())
}
