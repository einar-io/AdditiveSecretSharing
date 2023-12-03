use log::{debug, info};
use rand;
use std::{ops::Range, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinSet,
    time::sleep,
};

const PARTIES_COUNT: usize = 4;
const SALARIES: [i64; PARTIES_COUNT] = [160000, 180000, 190000, 210000];
const PARTIES: Range<usize> = 0..PARTIES_COUNT;

async fn party(this_party: usize) -> i64 {
    info!("party {this_party} started");
    let split = split(SALARIES[this_party]).await;
    info!("party {this_party} calculated split {split:#?}.");

    // phase one: exchange splits
    let mut nodes = JoinSet::new();
    for party in PARTIES {
        if party == this_party {
            nodes.spawn(sender(party, split));
        } else {
            nodes.spawn(receiver(party));
        }
    }

    let mut sum: i64 = 0;
    while let Some(part) = nodes.join_next().await {
        sum += part.unwrap();
    }
    info!("party {this_party} computed sum {sum}.");

    // phase two: share sums
    let mut nodes = JoinSet::new();
    for party in PARTIES {
        if party == this_party {
            nodes.spawn(sum_sender(party, sum));
        } else {
            nodes.spawn(sum_receiver(party));
        }
    }
    let mut sumsum = 0;
    while let Some(part) = nodes.join_next().await {
        sumsum += part.unwrap();
    }

    let avg_salary = sumsum / i64::try_from(PARTIES_COUNT).unwrap();
    info!("party {this_party} computed average salary of {avg_salary}.");
    // this is better/worse
    info!("party {this_party} terminating ..");
    avg_salary
}

async fn sender(this_party: usize, split: [i64; 4]) -> i64 {
    let port = 12121 + this_party;
    let addr = format!("127.0.0.1:{port}");
    debug!("Sender: {}", addr.clone());
    let listener = TcpListener::bind(addr).await.unwrap();
    for node in PARTIES {
        if node == this_party {
            continue;
        };
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await.unwrap();
        socket.writable().await.unwrap();
        debug!("Sending: {}", split[this_party]);
        let buf = bincode::serialize(&split[node]).unwrap();
        socket.try_write(&buf).unwrap();
    }
    split[this_party]
}

async fn receiver(other_party: usize) -> i64 {
    //info!("party {id} calculated split {split:#?}.");

    let port = 12121 + other_party;
    let addr = format!("127.0.0.1:{port}");
    debug!("Receiver: {}", addr.clone());
    sleep(Duration::from_secs(1)).await;
    let stream = TcpStream::connect(addr).await.unwrap();

    stream.readable().await.unwrap();
    let mut buf = vec![];
    stream.try_read_buf(&mut buf).unwrap();
    let res: i64 = bincode::deserialize(&buf).unwrap();
    debug!("Received: {}", res.clone());
    res
}

async fn sum_sender(this_party: usize, sum: i64) -> i64 {
    let port = 12121 + this_party;
    let addr = format!("127.0.0.1:{port}");
    debug!("Sender: {}", addr.clone());
    let listener = TcpListener::bind(addr).await.unwrap();
    for node in PARTIES {
        if node == this_party {
            continue;
        };
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await.unwrap();
        socket.writable().await.unwrap();
        debug!("Sending: {}", sum);
        let buf = bincode::serialize(&sum).unwrap();
        socket.try_write(&buf).unwrap();
    }
    sum
}

async fn sum_receiver(other_party: usize) -> i64 {
    //info!("party {id} calculated split {split:#?}.");

    let port = 12121 + other_party;
    let addr = format!("127.0.0.1:{port}");
    debug!("Receiver: {}", addr.clone());
    sleep(Duration::from_secs(1)).await;
    let stream = TcpStream::connect(addr).await.unwrap();

    stream.readable().await.unwrap();
    let mut buf = vec![];
    stream.try_read_buf(&mut buf).unwrap();
    let res: i64 = bincode::deserialize(&buf).unwrap();
    debug!("Received: {}", res.clone());
    res
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("hello, world!");
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

    info!("goodbye, world!")

}

async fn split(salary: i64) -> [i64; PARTIES_COUNT] {
    let mut split: [i64; PARTIES_COUNT] = [0; PARTIES_COUNT];

    let mut acc = 0;
    for id in 0..PARTIES_COUNT - 1 {
        split[id] = i64::from(rand::random::<i32>());
        acc += split[id];
    }

    *split.last_mut().unwrap() = salary - acc;

    assert_eq!(salary, split.iter().sum());
    split
}
