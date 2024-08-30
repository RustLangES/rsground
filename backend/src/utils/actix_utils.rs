use actix_web::HttpRequest;

pub fn get_base_url(req: HttpRequest) -> String {
    let cinfo = req.connection_info();

    format!(
        "{}://{}",
        cinfo.scheme(),
        cinfo.host()
    )
}
