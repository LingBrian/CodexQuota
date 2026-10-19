use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

async fn read_limited(reader: impl AsyncRead + Unpin, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "CLI output exceeds limit",
        ));
    }
    Ok(bytes)
}
pub async fn output(
    command: &mut tokio::process::Command,
    limit: usize,
) -> io::Result<std::process::Output> {
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let stdout = read_limited(
        child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("missing CLI pipe"))?,
        limit,
    )
    .await?;
    let status = child.wait().await?;
    Ok(std::process::Output {
        status,
        stdout,
        stderr: Vec::new(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn exact_limit_is_accepted_and_overflow_is_bounded() {
        assert_eq!(read_limited(&b"1234"[..], 4).await.unwrap(), b"1234");
        assert_eq!(
            read_limited(&b"12345678"[..], 4).await.unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
