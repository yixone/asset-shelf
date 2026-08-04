use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use tokio::io::{AsyncRead, ReadBuf};

/// Reader for an image processed using [`image`] crate
pub struct ImageReader {
    pub(crate) buffered: VecDeque<Bytes>,
}

impl AsyncRead for ImageReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }

        let inner_buf = match self.buffered.front_mut() {
            Some(c) => c,
            None => return Poll::Ready(Ok(())),
        };
        let len = inner_buf.len().min(buf.remaining());
        let to_put = inner_buf.split_to(len);
        buf.put_slice(&to_put);

        if inner_buf.is_empty() {
            self.buffered.pop_front();
        }

        Poll::Ready(Ok(()))
    }
}
