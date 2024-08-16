// use cookie_cutter::{encoding::Encoding, medium::Medium, SerializeBuf, SerializeIter};

// trait CRCProvider {
//     type Repr;
//     type Digest;

//     fn digest(&mut self) -> Self::Digest;
// }

// struct NoCRC; // provided

// impl CRCProvider for NoCRC {
//     type Repr = ();
//     type Digest = ();

//     fn digest(&mut self) -> Self::Digest {}
// }

// trait Packet<E: Encoding>: SerializeIter<E> {
//     type CRC: CRCProvider;

//     fn try_read(
//         buf: &mut <<Self as Serialize<E>>::Serialized as Medium<E>>::Unsized,
//         crc_provider: &mut Self::CRC,
//     );
//     fn write(&self, buf: &mut <Self as Serialize<E>>::Serialized);
// }

/*
#[derive(Packet)]
struct CommandPacket {
    // addr: PacketAddress,
    payload: Payload,
    #[crc]
    crc: CommandCRC,
}

impl<E: Encoding> Packet<E> for CommandPacket {
    fn try_read(buf: &mut <<Self as Serialize<E>>::Serialized as Medium<E>>::Unsized, crc_provider: &mut impl CRCProvider) -> Result<Self, SequenceError> {
        let digest = crc_provider.digest();

        let Segment { payload, used } = Serialize::deserialize(buf);

        digest.update(buf[..used]);

        let Segment { crc, used } = Serialize::deserialize(buf);

        if digest.finalize() == crc {
            Ok(
                Self {
                    cmd,
                    crc,
                }
            )
        } else {
            Err(SequenceError::CRCMismatch);
        }
    }

    fn write()
}
*/

// adc scratch pad

use std::marker::PhantomData;

trait AdcMode {}

struct Oneshot;
struct Continuous;

impl AdcMode for Oneshot {}
impl AdcMode for Continuous {}

trait AdcState {}

struct Shutdown;
struct Idle;
struct Active;

impl AdcState for Shutdown {}
impl AdcState for Idle {}
impl AdcState for Active {}

struct Adc<Adc, State: AdcState, Mode: AdcMode, Conversion> {
    rb: Adc,
    _state: PhantomData<State>,
    _mode: PhantomData<Mode>,
    _conversion: PhantomData<Conversion>,
}

// watchdog?

// fn get_conversion(&mut self) -> Result
// async fn wait_for_conversion(&mut self) -> ?
