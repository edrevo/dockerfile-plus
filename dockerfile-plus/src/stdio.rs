use std::io::{self, stdin, stdout, Read, Write};
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use std::{
    io::{Stdin, Stdout},
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use tokio::io::*;
use tonic::transport::{server::Connected, Uri};

#[pin_project]
pub struct StdioSocket<R: Read + AsRawFd, W: Write + AsRawFd> {
    #[pin]
    reader: PollEvented<async_stdio::EventedStdin<R>>,

    #[pin]
    writer: PollEvented<async_stdio::EventedStdout<W>>,
}

pub async fn stdio_connector(_: Uri) -> io::Result<StdioSocket<Stdin, Stdout>> {
    StdioSocket::try_new()
}

impl StdioSocket<Stdin, Stdout> {
    pub fn try_new() -> io::Result<Self> {
        Self::try_new_rw(stdin(), stdout())
    }
}

impl<R: Read + AsRawFd, W: Write + AsRawFd> Connected for StdioSocket<R, W> {
    fn remote_addr(&self) -> Option<SocketAddr> {
        Some(SocketAddr::new(IpAddr::from(Ipv4Addr::UNSPECIFIED), 8080))
    }
}

impl<R: Read + AsRawFd, W: Write + AsRawFd> StdioSocket<R, W> {
    pub fn try_new_rw(read: R, write: W) -> io::Result<Self> {
        Ok(StdioSocket {
            reader: PollEvented::new(async_stdio::EventedStdin::try_new(read)?)?,
            writer: PollEvented::new(async_stdio::EventedStdout::try_new(write)?)?,
        })
    }
}

impl<R: Read + AsRawFd + Unpin, W: Write + AsRawFd + Unpin> AsyncRead for StdioSocket<R, W> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        self.project().reader.poll_read(cx, buf)
    }
}

impl<R: Read + AsRawFd + Unpin, W: Write + AsRawFd + Unpin> AsyncWrite for StdioSocket<R, W> {
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

mod async_stdio {
    use std::io::{self, Read, Write};
    use std::os::unix::io::AsRawFd;

    use mio::event::Evented;
    use mio::unix::EventedFd;
    use mio::{Poll, PollOpt, Ready, Token};

    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

    pub struct EventedStdin<T: Read + AsRawFd>(T);
    pub struct EventedStdout<T: Write + AsRawFd>(T);

    impl<T: Read + AsRawFd> EventedStdin<T> {
        pub fn try_new(stdin: T) -> io::Result<Self> {
            set_non_blocking_flag(&stdin)?;

            Ok(EventedStdin(stdin))
        }
    }

    impl<T: Write + AsRawFd> EventedStdout<T> {
        pub fn try_new(stdout: T) -> io::Result<Self> {
            set_non_blocking_flag(&stdout)?;

            Ok(EventedStdout(stdout))
        }
    }

    impl<T: Read + AsRawFd> Evented for EventedStdin<T> {
        fn register(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).register(poll, token, interest, opts)
        }

        fn reregister(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &Poll) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).deregister(poll)
        }
    }

    impl<T: Read + AsRawFd> Read for EventedStdin<T> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl<T: Write + AsRawFd> Evented for EventedStdout<T> {
        fn register(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).register(poll, token, interest, opts)
        }

        fn reregister(
            &self,
            poll: &Poll,
            token: Token,
            interest: Ready,
            opts: PollOpt,
        ) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &Poll) -> io::Result<()> {
            EventedFd(&self.0.as_raw_fd()).deregister(poll)
        }
    }

    impl<T: Write + AsRawFd> Write for EventedStdout<T> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }

    fn set_non_blocking_flag<T: AsRawFd>(stream: &T) -> io::Result<()> {
        let flags = unsafe { fcntl(stream.as_raw_fd(), F_GETFL, 0) };

        if flags < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if unsafe { fcntl(stream.as_raw_fd(), F_SETFL, flags | O_NONBLOCK) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }
}
