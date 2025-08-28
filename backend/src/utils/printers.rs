use core::fmt;

pub struct Truncated<const LEN: usize, T: Sized>(T);

impl<const LEN: usize, T: fmt::Debug> fmt::Debug for Truncated<LEN, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buf = format!("{:?}", self.0);
        if buf.len() > LEN {
            f.write_str(&buf[0..LEN])?;
            f.write_str("...")
        } else {
            f.write_str(&buf)
        }
    }
}

impl<const LEN: usize, T: fmt::Display> fmt::Display for Truncated<LEN, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buf = format!("{}", self.0);
        if buf.len() > LEN {
            f.write_str(&buf[0..LEN])?;
            f.write_str("...")
        } else {
            f.write_str(&buf)
        }
    }
}

pub trait Truncate: Sized {
    fn truncate<const LEN: usize>(self) -> Truncated<LEN, Self> {
        Truncated(self)
    }
}

impl<T> Truncate for T {}
