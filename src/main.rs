use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

async fn my_async_fn() {
    println!("Hello form async");
    let _socket = TcpStream::connect("127.0.0.1:3000").await.unwrap();

    // await will wait for complete the executuin in the mean it it's allow other other operiaont
    // if the connection is not establish then panic finished the programm.
    println!("async TCP operation complete");
}

struct Delay {
    when: Instant,
}

// pub trait Future {
//     type Output;
//     // The associated type Output will produce once it completes.
//     fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
//     // `Pin<&mut Self>` in aynchronous context it's essential mutable borrow to prevent data race and memory safety.
//     // Pin type is used to indicate that data at the given memory location should not be move. ensure future state not unintentional change.
// }

impl Future for Delay {
    type Output = &'static str;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            //Ignore this line for now
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// Implementing Future on a Enum above impl on Struct

enum MainFuture {
    State0,
    State1(Delay),
    Terminated,
}

impl Future for MainFuture {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use MainFuture::*;
        loop {
            match *self {
                State0 => {
                    let when = Instant::now() + Duration::from_millis(10);
                    let future = Delay { when };
                    *self = State1(future);
                }
                State1(ref mut my_future) => match Pin::new(my_future).poll(cx) {
                    Poll::Ready(out) => {
                        assert_eq!(out, "done");
                        *self = Terminated;
                        return Poll::Ready(());
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                },
                Terminated => {
                    panic!("Futue polled after completion")
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when };
    let out = future.await;
    assert_eq!(out, "done");
}
