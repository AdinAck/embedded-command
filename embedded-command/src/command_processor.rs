use core::{future::Future, marker::PhantomData};
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Receiver};
use embedded_io_async::{BufRead, ErrorType, Write};
use heapless::Vec;

type CommandBuffer<const N: usize> = Vec<u8, N>;

pub trait CommandBundle {}

enum Response {
    Ok,
}

enum Packet<Bundle: CommandBundle> {
    Command(Bundle),
    Response(Response),
}

impl<Bundle: CommandBundle> From<Bundle> for Packet<Bundle> {
    fn from(value: Bundle) -> Self {
        Self::Command(value)
    }
}

impl<Bundle: CommandBundle> From<Response> for Packet<Bundle> {
    fn from(value: Response) -> Self {
        Self::Response(value)
    }
}

pub struct CommandProcessor<Port, Bundle, const N: usize>
where
    Port: BufRead + Write,
    Bundle: CommandBundle,
{
    port: Port,
    buf: CommandBuffer<N>,
    _p: PhantomData<Bundle>,
}

impl<Port, Bundle, const N: usize> CommandProcessor<Port, Bundle, N>
where
    Port: BufRead + Write,
    Bundle: CommandBundle,
{
    pub const fn new(port: Port) -> Self {
        Self {
            port,
            buf: CommandBuffer::new(),
            _p: PhantomData,
        }
    }

    async fn poll(&mut self) -> Result<(), <Port as ErrorType>::Error> {
        Ok(())
    }

    fn try_parse(&mut self) -> Result<Bundle, ()> {
        todo!()
    }

    async fn try_receive(&mut self) -> Bundle {
        loop {
            if let Err(_) = self.poll().await {
                // do something
            }

            if let Ok(bundle) = self.try_parse() {
                break bundle;
            }
        }
    }

    async fn send(&mut self, packet: Packet<Bundle>) {
        todo!()
    }

    pub async fn run<'a, M, F, Fut, const K: usize>(
        &mut self,
        send_queue: Receiver<'a, M, Bundle, K>,
        mut on_dispatch: F,
    ) where
        M: RawMutex,
        F: FnMut(Bundle) -> Fut,
        Fut: Future<Output = ()>,
    {
        loop {
            match select(send_queue.receive(), self.try_receive()).await {
                Either::First(cmd) => {
                    // command to send has been received from the queue

                    // send command
                    self.send(cmd.into()).await;

                    // wait for response or timeout
                }
                Either::Second(bundle) => {
                    // a command has been received from the bus

                    // if valid, dispatch it
                    on_dispatch(bundle).await;

                    // respond with status
                    self.send(Response::Ok.into()).await;
                }
            }
        }
    }
}
