use futures::stream;

pub trait ToStream<I> {
    fn to_stream(self) -> stream::Iter<I>;
}

impl<I: IntoIterator> ToStream<I::IntoIter> for I {
    fn to_stream(self) -> stream::Iter<I::IntoIter> {
        stream::iter(self)
    }
}
