use serde::Serialize;

pub trait SessionExt {
    /// Serialize data to json and send it as text
    async fn text_json<T>(&mut self, data: &T) -> Result<(), actix_ws::Closed>
    where
        T: ?Sized + Serialize;
}

impl SessionExt for actix_ws::Session {
    async fn text_json<T>(&mut self, data: &T) -> Result<(), actix_ws::Closed>
    where
        T: ?Sized + Serialize,
    {
        let text = serde_json::to_string(data).unwrap();
        self.text(text).await
    }
}
