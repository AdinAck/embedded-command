#![no_std]

pub mod command_buffer;
pub mod command_processor;
pub mod crc;

struct CommandBundle;

enum Response {
    Ok,
}

enum Payload {
    Command(CommandBundle),
    Response(Response),
}

struct Transaction {
    payload: Payload,
    crc: u16,
}
