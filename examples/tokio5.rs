use anyhow::Result;
use tokio::{
    runtime::Builder,
    time::{sleep, Duration},
};

// fn main() -> Result<()> {
//     let rt = Runtime::new()?;

//     rt.block_on(async {
//         println!("signal Runtime start");

//         for i in 0..3 {
//             println!("task {} ", i);
//             sleep(Duration::from_millis(500)).await;
//         }
//         println!("finish");
//     });

//     Ok(())
// }

// fn main() -> Result<()> {
//     let rt = Builder::new_multi_thread()
//         .worker_threads(4)
//         .thread_name("tokio-worker")
//         .enable_all()
//         .build()?;

//     rt.block_on(async {
//         println!("runtime start");

//         let tasks = (1..=5).map(|i| {
//             tokio::spawn(async move {
//                 println!("task {} on {:?}", i, thread::current().id());
//                 sleep(Duration::from_millis(1000)).await;
//                 println!("task {} finish", i);
//             })
//         });

//         for task in tasks {
//             task.await.unwrap();
//         }
//     });
//     println!("finish");

//     Ok(())
// }

// fn main() -> Result<()> {
//     let rt = Builder::new_multi_thread()
//         .worker_threads(2)
//         .max_blocking_threads(4)
//         .thread_name("my-tokio")
//         .thread_stack_size(3 * 1024 * 1024)
//         .enable_time()
//         .enable_io()
//         .build()?;

//     rt.block_on(async {
//         let start = Instant::now();

//         let cpu_task = task::spawn_blocking(|| {
//             println!("CPU start");
//             let mut sum: u64 = 0;
//             for i in 0..1_000_000 {
//                 sum += i;
//             }
//             println!("CPU finish, ret: {}", sum);
//             sum
//         });

//         let io_task = async {
//             println!("io start");
//             sleep(Duration::from_millis(500)).await;
//             println!("io finish");
//             "io ret"
//         };

//         let (cpu_ret, io_ret) = tokio::join!(cpu_task, io_task);
//         println!(
//             "cpu ret: {:?}, io ret: {}, all take: {}",
//             cpu_ret,
//             io_ret,
//             start.elapsed().as_secs_f64()
//         );
//     });

//     Ok(())
// }

// 当前线程
fn main() -> Result<()> {
    let rt = Builder::new_current_thread().enable_all().build()?;

    rt.block_on(async {
        println!("current thread runtime start");

        let task1 = async {
            println!("task1 start");
            sleep(Duration::from_millis(100)).await;
            println!("task1 finish");
        };

        let task2 = async {
            println!("task2 start");
            sleep(Duration::from_millis(100)).await;
            println!("task2 finish");
        };

        tokio::join!(task1, task2);

        println!("current thread runtime finish");
    });

    Ok(())
}
