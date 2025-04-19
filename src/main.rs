use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct Job {
    id: i32,
    value: i32,
}

fn worker(
    id: usize,
    jobs: Arc<Mutex<mpsc::Receiver<Option<Job>>>>,
    job_tx: mpsc::Sender<Option<Job>>,
    result_tx: mpsc::Sender<Job>,
) {
    loop {
        // recv() an Option<Job>
        let job_opt = { jobs.lock().unwrap().recv().unwrap() };
        match job_opt {
            Some(mut job) => {
                println!("{}{}:{}:{}", "        ".repeat(id), id, job.id, job.value);
                thread::sleep(Duration::from_secs(1));

                if job.value >= 10 {
                    result_tx.send(job).unwrap();
                } else {
                    job.value += 1;
                    job_tx.send(Some(job)).unwrap();
                }
            }
            None => {
                // shutdown signal
                break;
            }
        }
    }
}

fn main() {
    let arr = [3, 5, 11, 10, 7, 2, 8, 8, 8, 8, 8, 8, 8];
    let (job_tx, job_rx) = mpsc::channel::<Option<Job>>();
    let (result_tx, result_rx) = mpsc::channel();

    let job_rx = Arc::new(Mutex::new(job_rx));
    let mut handles = Vec::new();
    let n_workers = 4;

    for i in 1..=n_workers {
        let jobs_clone = Arc::clone(&job_rx);
        let job_tx_clone = job_tx.clone();
        let result_tx_clone = result_tx.clone();
        handles.push(thread::spawn(move || {
            worker(i, jobs_clone, job_tx_clone, result_tx_clone);
        }));
    }

    // send the real jobs
    for (i, &v) in arr.iter().enumerate() {
        job_tx
            .send(Some(Job {
                id: (i + 1) as i32,
                value: v,
            }))
            .unwrap();
    }

    let mut results = Vec::new();
    for _ in 0..arr.len() {
        let r = result_rx.recv().unwrap();
        println!("Job ID: {}, Final Value: {}", r.id, r.value);
        results.push(r);
    }

    // now tell each worker to shut down
    for _ in 0..n_workers {
        job_tx.send(None).unwrap();
    }

    // drop our last sender so channels clean up
    drop(job_tx);

    // join them
    for h in handles {
        h.join().unwrap();
    }
    println!("All workers have completed.");
}
