// To run asynchronous functions, must either be passed to tokio::spawn or be the main function annoted with #[tokio::main]

use futures::task;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};
/*
task form futures crates

*/

struct Delay {
    when: Instant
}
impl Future for Delay {
    type Output = &'static str;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        }else {
            let waker = cx.waker().clone();
            let when = self.when;
            // spawn a timer thread
            thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    thread::sleep(when - now)
                }
                waker.wake();

            });
            Poll::Pending
        }
    }
    
}
type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
struct MiniTokio {
    tasks: VecDeque<Task>,
}

impl MiniTokio {
    fn new() -> MiniTokio {
        MiniTokio {
            tasks: VecDeque::new(),
        }
    }
    // to run asynchronously we put code spawn
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(future))
    }

    fn run(&mut self) {
        let waker = task::noop_waker() ; // task import form future crate
        // waker is Waker instance need to .wake() to call on it.
        let mut cx = Context::from_waker(&waker);
        // return a reference of current waker for the current task

        while let Some(mut task) = self.tasks.pop_front()  {
            if task.as_mut().poll(&mut cx).is_pending(){
                self.tasks.push_back(task);
            }
        }


    }
}
fn main() {
    let mut mini_tokio = MiniTokio::new();
    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay{when};
        let out = future.await;
        assert_eq!(out, "done");
    });
    mini_tokio.run();
}
