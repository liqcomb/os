#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    EFAIL,
    ENOMEM,
    EFAULT,
    EFULL,
    ENOENT,
    EAGAIN,
    EIO,
    EBADFS,
    EINVAL,
}

impl Error {
    pub fn message(&self) -> &'static str {
        match self {
            Error::EFAIL => "Operation failed",
            Error::ENOMEM => "Insufficient memory",
            Error::EFAULT => "Access violation",
            Error::EFULL => "No more space",
            Error::ENOENT => "Target not found",
            Error::EAGAIN => "Already exist",
            Error::EIO => "I/O Error",
            Error::EBADFS => "Bad filesystem",
            Error::EINVAL => "Invalid argument",
            _ => "Uncategorized error",
        }
    }
}
