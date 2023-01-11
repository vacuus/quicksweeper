pub mod socket_pc;
pub mod socket_web;

pub enum CommonConnection {
    ConnectionDesktop(socket_pc::Connection),
    ConnectionWeb(socket_web::Connection),
}

impl CommonConnection {

}