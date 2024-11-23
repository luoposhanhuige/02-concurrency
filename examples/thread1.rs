use anyhow::Result;
use std::{sync::mpsc, thread, time::Duration};

const NUM_PRODUCERS: usize = 4;

#[allow(dead_code)]
// The #[allow(dead_code)] attribute in Rust is used to suppress compiler warnings about unused code. Specifically, it tells the Rust compiler to ignore warnings for code that is defined but not used anywhere in the program. This can be useful during development when you have functions, variables, or other items that are not yet used but you want to keep them in the codebase without triggering warnings.
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // create several producers thread
    // the producers thread will run in parallel, each sending a message to the receiver every time duration.
    for i in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx)); // 1、创建线程的一种方式，直接用函数作为线程的主函数。
    }
    drop(tx); // drop the original sender, so that the receiver will not wait for more messages.
              // drop 函数会立即释放 tx 的所有权，所以，rx 不会等待更多的消息。

    // create a consumer thread
    // we need to have the result from the thread::spawn function to be able to join the consumer thread.
    let consumer = thread::spawn(move || {
        // 2、创建线程的另一种方式，用闭包作为线程的主函数。
        for msg in rx {
            println!("Received: {:?}", msg);
        }
        println!("Consumer exiting"); // 主要与上述 drop(tx) 响应，当 tx 被 drop 时，rx 会收到一个 None，所以，当 rx 收到 None 时，就会退出循环。
        "end" // 也可以是一个数字什么的，诸如888。作为一个返回值，给到 consumer.join()。
    });

    // join the consumer thread
    // consumer.join().expect("consumer thread panicked");
    // consumer.join().unwrap();
    // consumer.join()?; // 将报错，因为 Result 类型没有实现对 joinhandle 的解引用
    // The reason for using thread::spawn to create a consumer thread is to demonstrate how to create and join threads, but it is not strictly necessary if you are fine with handling the messages in the main thread.
    // 如果不使用 join()，那么主线程main()会直接退出，不会等待其他线程执行完，导致其他线程也会退出。所以，必须使用 join() 来等待其他线程结束。
    let secret = consumer
        .join()
        .map_err(|e| anyhow::anyhow!("Thread join error: {:?}", e))?; //因为 e 实现了 debug，所以，用 {:?} 输出。这里是用 anyhow::anyhow! 宏来创建一个 anyhow::Error 类型的错误。
                                                                      // 这里是一个技巧，当我们没办法通过 "?" 把一种错误转换为另一种错误时，我们可以用 map_err 函数来转换错误类型。

    // consumer.join()只是等待线程结束，但是不会返回线程的返回值，所以，如果我们想要获取线程的返回值，我们可以使用线程的返回值，也就是 JoinHandle 类型的实例。
    // q: consumer 的线程中，对 rx 进行了迭代，但没有对 tx 进行任何处理，但 consumer.join() 会等待 tx 的线程结束，这是为什么？
    // a: 因为 tx 是一个 Sender 类型的实例，它是一个生产者，而 rx 是一个 Receiver 类型的实例，它是一个消费者。tx 和 rx 是一对一的关系，所以，当 tx 的线程结束时，rx 的线程也会结束。所以，consumer.join() 会等待 tx 的线程结束。

    // Use the receiver directly in the main thread
    // for msg in rx {
    //     println!("Received: {:?}", msg);
    // }
    // print secret
    println!("Secret: {:?}", secret);

    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>(); // cargo add rand
        tx.send(Msg::new(idx, value))?; // "?" is a shorthand for error handling that either returns the value inside the Result if it is Ok, or returns the error if it is Err.
                                        // let sleep_time = rand::random::<u8>() * 10; // 会造成溢出，因为 u8 的最大值是 255，255 * 10 = 2550，而 u8 的最大值是 255，所以，这里会造成溢出。
                                        // thread::sleep(Duration::from_millis(sleep_time as u64)); // 或者 “sleep_time as _”，rust compiler repulsively requires the type conversion.
                                        //the thread will sleep for 1000 milliseconds毫秒, which is equivalent to 1 second.
        let sleep_time = rand::random::<u8>() as u64 * 10;
        thread::sleep(Duration::from_millis(sleep_time));
        // random exit the producer
        if rand::random::<u8>() % 5 == 0 {
            println!("Producer {} exiting", idx);
            // break; // 因为要退出循环，所以，需要一个返回值，这里只是 break，就是报错。
            return Ok(()); // 或者在 loop 之后，返回一个 Ok(())，表示正常退出。
        }
    }
}
// In this case, the thread will sleep for 1 second after the tx.send(value)? operation. The thread::sleep function is called after the message is sent, so the thread will pause for 1 second before continuing with the next iteration of the loop or any subsequent code. This will slow down the rate at which messages are sent to the receiver, which will help us see the messages being sent and received more clearly.

impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}
