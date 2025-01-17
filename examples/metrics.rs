use anyhow::Result;
use concurrency::Metrics;
use rand::Rng;
use std::{thread, time::Duration};

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    let metrics = Metrics::new();

    println!("{:?}", metrics.snapshot()); // prints the data wrapped in the Arc<Mutex<HashMap<String, i64>>> which is an empty HashMap

    for idx in 0..N {
        task_worker(idx, metrics.clone())?; // deep copy, Metrics{data: Arc::clone(&metrics.data)}
    }

    for _ in 0..M {
        request_worker(metrics.clone())?;
    }

    loop {
        thread::sleep(Duration::from_secs(2));
        println!("{:?}", metrics.snapshot());
    }

    // Ok(())
}

fn task_worker(idx: usize, metrics: Metrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            // do long term stuff
            // rand::thread_rng() creates a random number generator (RNG) that is local to the current thread.
            // This RNG is seeded by the operating system and is safe to use in a multi-threaded context.
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..5000))); // 0.1s ~ 5s

            metrics.inc(format!("call.thread.worker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(()) // use anyhow::{Ok, Result}; 会报错，参数不对，，因为 anyhow::Error 不是 std::error::Error 的子类
    });

    Ok(())
}

fn request_worker(metrics: Metrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            // process requests
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(50..800))); // 0.05s ~ 0.8s
            let page = rng.gen_range(1..5);

            // "?" operator can only be used in the closure that returns Result or Option
            metrics.inc(format!("req.page.{}", page))?; // metrics.inc(format!("req.page.{}", page)).unwrap(); use ? instead of unwrap to propagate the error
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(()) // 因为上面是 loop，所以这里永远不会执行到；但如果没有这一行，编译器会报错，因为上面用了 ?，所以这里需要返回一个 Result。
    });

    Ok(())
}
