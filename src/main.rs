use log::{debug, info};
use std::{cmp::Ordering, ops::Range, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinSet,
    time::sleep,
};

const COUNT: usize = 4;
const SALARIES: [i64; COUNT] = [180000, 220000, 220000, 260000];
const PARTIES: Range<usize> = 0..COUNT;

async fn party(this_party: usize) -> i64 {
    info!("<Party {this_party}> started");
    let split = split(SALARIES[this_party]).await;
    info!("<Party {this_party}> split his salary into random fragments: {split:#?}.");

    // phase one: exchange splits
    let mut nodes = JoinSet::new();
    for party in PARTIES {
        if party == this_party {
            nodes.spawn(sender(party, split));
        } else {
            nodes.spawn(receiver(this_party, party));
        }
    }

    let mut sum: i64 = 0;
    while let Some(part) = nodes.join_next().await {
        sum += part.unwrap();
    }
    info!("<Party {this_party}> computed sum {sum}.");

    // phase two: share sums
    let mut nodes = JoinSet::new();
    for party in PARTIES {
        if party == this_party {
            nodes.spawn(sum_sender(party, sum));
        } else {
            nodes.spawn(sum_receiver(this_party, party));
        }
    }
    let mut party_sum = 0;
    while let Some(part) = nodes.join_next().await {
        party_sum += part.unwrap();
    }

    let avg_salary = party_sum / i64::try_from(COUNT).unwrap();
    info!("<Party {this_party}> computed average salary of {avg_salary}.");
    // this is better/worse
    let my_salary = SALARIES[this_party];
    match my_salary.cmp(&avg_salary) {
        Ordering::Greater => {
            info!("<Party {this_party}> learns that he earns exactly the average salary 🟡.")
        }
        Ordering::Less => info!(
            "<Party {this_party}> learns that he earns {} more than the average salary ✅.",
            my_salary - avg_salary
        ),
        Ordering::Equal => info!(
            "<Party {this_party}> learns that he earns {} less than the average salary ❌.",
            my_salary - avg_salary
        ),
    }

    info!("<Party {this_party}> terminating ..");
    avg_salary
}

async fn sender(this_party: usize, split: [i64; COUNT]) -> i64 {
    let port = 12121 + this_party;
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(addr).await.unwrap();
    for node in PARTIES {
        if node == this_party {
            continue;
        };
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await.unwrap();
        socket.writable().await.unwrap();
        debug!("<Party {this_party}> sent fragment: {}", split[this_party]);
        let buf = bincode::serialize(&split[node]).unwrap();
        socket.try_write(&buf).unwrap();
    }
    split[this_party]
}

async fn receiver(this_party: usize, other_party: usize) -> i64 {
    let port = 12121 + other_party;
    let addr = format!("127.0.0.1:{port}");

    let stream;
    loop {
        sleep(Duration::from_secs(1)).await;
        if let Ok(stream_) = TcpStream::connect(addr.clone()).await {
            stream = stream_;
            break;
        }
    }

    stream.readable().await.unwrap();
    let mut buf = vec![];
    stream.try_read_buf(&mut buf).unwrap();
    let res: i64 = bincode::deserialize(&buf).unwrap();
    debug!("<Party {this_party}> received from <Party {other_party}> fragment: {res}",);
    res
}

async fn sum_sender(this_party: usize, sum: i64) -> i64 {
    let port = 12121 + this_party;
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(addr).await.unwrap();
    for node in PARTIES {
        if node == this_party {
            continue;
        };
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await.unwrap();
        socket.writable().await.unwrap();
        debug!("<Party {this_party}> sent sum: {}", sum);
        let buf = bincode::serialize(&sum).unwrap();
        socket.try_write(&buf).unwrap();
    }
    sum
}

async fn sum_receiver(this_party: usize, other_party: usize) -> i64 {
    let port = 12121 + other_party;
    let addr = format!("127.0.0.1:{port}");

    let stream;
    loop {
        sleep(Duration::from_secs(1)).await;
        if let Ok(stream_) = TcpStream::connect(addr.clone()).await {
            stream = stream_;
            break;
        }
    }

    stream.readable().await.unwrap();
    let mut buf = vec![];
    stream.try_read_buf(&mut buf).unwrap();
    let res: i64 = bincode::deserialize(&buf).unwrap();
    debug!("<Party {this_party}> received from <Party {other_party}> sum: {res}.",);
    res
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("<main> Hello, world!");
    let mut nodes = JoinSet::new();
    for party_id in PARTIES {
        nodes.spawn(party(party_id));
    }
    let numerator = SALARIES.iter().sum::<i64>();
    let denominator = i64::try_from(SALARIES.len()).unwrap();
    let avg_actual = numerator / denominator;
    while let Some(avg) = nodes.join_next().await {
        assert_eq!(avg.unwrap(), avg_actual);
    }
    info!("<main> Goodbye, world!")
}

async fn split(salary: i64) -> [i64; COUNT]{
    let mut split: [i64; COUNT] = [0; COUNT];

    let mut acc = 0;
    for item in split.iter_mut().take(COUNT - 1) {
        *item = i64::from(rand::random::<i32>());
        acc += *item;
    }

    *split.last_mut().unwrap() = salary - acc;

    debug_assert_eq!(salary, split.iter().sum());
    split
}
