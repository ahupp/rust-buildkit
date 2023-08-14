use std::io::{self};
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper_util::rt::TokioIo;
use pin_project::pin_project;
use std::io::Result;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tonic::transport::Uri;

#[pin_project]
pub struct StdioSocket {
    #[pin]
    reader: tokio::io::Stdin,

    #[pin]
    writer: tokio::io::Stdout,
}

pub async fn stdio_connector(_: Uri) -> io::Result<TokioIo<StdioSocket>> {
    Ok(TokioIo::new(StdioSocket {
        reader: tokio::io::stdin(),
        writer: tokio::io::stdout(),
    }))
}

impl AsyncRead for StdioSocket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        self.project().reader.poll_read(cx, buf)
    }
}

impl AsyncWrite for StdioSocket {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.project().writer.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().writer.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().writer.poll_shutdown(cx)
    }
}
