use crate::types::Html;

pub const YAML_FILE: &str = "/var/www/html/data/shroom_info.yaml";

pub const IMAGE_DIR: &str = "/var/www/html/mycology/Smallimages";

pub const LOG_FILE: &str = "/home/debbyadmin/gen_shroom_info.log";

pub const HOST_NAME: &str = "muon";

pub mod domains {
    pub const NO_DOMAIN: &str = "";
    pub const MYCOLOGY: &str = "mycology";
}

pub mod status {
    pub const HTTP_200: &str = "HTTP/1.1 200 OK";
    pub const HTTP_404: &str = "HTTP/1.1 404 NOT FOUND";
    pub const HTTP_403: &str = "HTTP/1.1 403 FORBIDDEN";
}

pub const PATH: Html = Html {
    root: "/var/www/html",
    nf404: "/var/www/html/data/404.html",
    pd403: "/var/www/html/data/403.html",
    meta: "/var/www/html/data/meta.html",
    menu: "/var/www/html/data/menu.html",
    shroompage: "/var/www/html/data/page.html",
};

pub const MIMETYPES: [(&str, &str); 6] = [
    ("jpg", "image/jpeg"),
    ("png", "image/png"),
    ("html", "text/html"),
    ("txt", "text/plain"),
    ("css", "text/css"),
    ("xml", "application/xml"),
];
