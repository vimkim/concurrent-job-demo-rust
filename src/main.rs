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
    jobs: Arc<Mutex<mpsc::Receiver<Job>>>,
    job_tx: mpsc::Sender<Job>,
    result_tx: mpsc::Sender<Job>,
) {
    loop {
        // Retrieve a job from the channel.
        // Locking is required because the Rust std receiver isn’t multi-consumer.
        let job_result = {
            let lock = jobs.lock().unwrap();
            lock.recv()
        };

        // Exit the loop if the channel is closed.
        let mut job = match job_result {
            Ok(job) => job,
            Err(_) => break,
        };

        // Print an indented status similar to Go's: worker id, job id, and current job value.
        println!("{}{}:{}:{}", "        ".repeat(id), id, job.id, job.value);
        thread::sleep(Duration::from_secs(1));

        if job.value >= 10 {
            // Job is complete; send it to the results channel.
            result_tx.send(job).unwrap();
        } else {
            // Increment job value and requeue.
            job.value += 1;
            job_tx.send(job).unwrap();
        }
    }
}

fn main() {
    // Define the array of job initial values.
    let arr = [3, 5, 11, 10, 7, 2, 8, 8, 8, 8, 8, 8, 8];

    // Create channels: one for jobs (with multiple senders) and one for results.
    let (job_tx, job_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    // Wrap the job receiver in an Arc<Mutex> to share among threads.
    let job_rx = Arc::new(Mutex::new(job_rx));

    // Spawn 4 worker threads.
    let mut handles = Vec::new();
    for i in 1..=4 {
        let jobs_clone = Arc::clone(&job_rx);
        let job_tx_clone = job_tx.clone();
        let result_tx_clone = result_tx.clone();
        let handle = thread::spawn(move || {
            worker(i, jobs_clone, job_tx_clone, result_tx_clone);
        });
        handles.push(handle);
    }

    // Send the initial set of jobs.
    for (i, &val) in arr.iter().enumerate() {
        let job = Job {
            id: (i + 1) as i32,
            value: val,
        };
        job_tx.send(job).unwrap();
    }

    // Collect jobs that have reached a value of at least 10.
    let mut result_arr = Vec::with_capacity(arr.len());
    for _ in 0..arr.len() {
        let result = result_rx.recv().unwrap();
        println!("Job ID: {}, Final Value: {}", result.id, result.value);
        result_arr.push(result);
    }

    // After all expected results are received,
    // drop the job sender to signal workers to exit once their channel is empty.
    drop(job_tx);

    println!("All jobs have been sent.");

    // Join all worker threads.
    for handle in handles {
        handle.join().unwrap();
    }

    println!("All workers have completed.");

    // Output the final results.
    for (i, res) in result_arr.iter().enumerate() {
        println!("Result {}: ID: {}, Value: {}", i + 1, res.id, res.value);
    }
}
