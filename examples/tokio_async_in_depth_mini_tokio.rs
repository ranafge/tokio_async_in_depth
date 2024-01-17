// To run asynchronous functions, must either be passed to tokio::spawn or be the main function annoted with #[tokio::main]


use futures::future::poll_fn;
use futures::lock::Mutex;
use futures::task::ArcWake;
use futures::task;
use tokio::sync::Notify;
use tokio::sync::futures::Notified;
// use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, mpsc};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};
/*
task form futures crates

*/

struct Delay {
    when: Instant,
    waker: Option<Arc<Mutex<Waker>>>,
}
impl Future for Delay {
    type Output = &'static str;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Hello world");
            return  Poll::Ready("done")
        }
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            if waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone()
            }
        }else {

            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());
            // spawn a timer thread
            thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    thread::sleep(when - now)
                }
               let waker = waker.lock().unwrap();
               waker.wake_by_ref();

            });
            Poll::Pending
        }
    }
    
}
// type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

struct Task {
    future : Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: mpsc::Sender<Arc<Task>>
}

impl Task {
    fn poll(self:Arc<Self>){
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut future = self.future.try_lock().unwrap();

        let _ = future.as_mut().poll(&mut cx);
    }

    fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>) 
    where 
        F: Future<Output = ()> + Send + 'static,
        {
            let task = Arc::new(Task{
                future: Mutex::new(Box::pin(future)),
                executor: sender.clone(),
            });
            let _ = sender.send(task);
        }
}
// Task struct as a waker

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.scheduled()
    }
}


// struct mintokio
struct MiniTokio {
    // tasks: VecDeque<Task>,
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>
}

impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = mpsc::channel();
        MiniTokio {
          scheduled, sender}
        }
    
    // to run asynchronously we put code spawn
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
    
    }

    fn run(&mut self) {
        // let waker = task::noop_waker() ; // task import form future crate
        // // waker is Waker instance need to .wake() to call on it.
        // let mut cx = Context::from_waker(&waker);
        // // return a reference of current waker for the current task

        // while let Some(mut task) = self.tasks.pop_front()  {
        //     if task.as_mut().poll(&mut cx).is_pending(){
        //         self.tasks.push_back(task);
        //     }
        // }

        while let Ok(task) = self.scheduled.recv()  {
            task.poll();
        }


    }
}

#[tokio::main]
async fn main() {
    // let mut mini_tokio = MiniTokio::new();
    // mini_tokio.spawn(async {
    //     let when = Instant::now() + Duration::from_millis(10);
    //     let future = Delay{when};
    //     let out = future.await;
    //     assert_eq!(out, "done");
    // });
    // mini_tokio.run();

    let when = Instant::now() + Duration::from_millis(10);
    let mut delay = Some(Delay {when});

    poll_fn(move |cx|{
        let mut delay = delay.take().unwrap();
        let res = Pin::new(&mut delay).poll(cx);
        assert!(res.is_pending());
        tokio::spawn(async move {
            delay.await
        });
        Poll::Ready(())
    }).await
}


async fn delay(dur: Duration)  {
    let when = Instant::now() + dur;
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    thread::spawn(move || {
        let now = Instant::now();
        if now < when {
            thread::sleep(when - now)
        }
        notify_clone.notify_one();
    });

    notify.notified().await;
}