pub trait AsyncInto<T> {
    async fn async_into(self) -> T;
}
