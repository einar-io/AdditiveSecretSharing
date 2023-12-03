use log::info;
use tokio::net::{TcpListener, TcpStream};
use rand;
use futures::future::{self, Ready};


const parties_count: usize = 4;
const salaries: [isize; parties_count] = [160000, 180000, 190000, 210000];

#[derive(Clone)]
struct Party;

#[tarpc::service]
trait AdditiveSecretSharing {
    async fn start(id: isize);
    async fn stop();
    async fn share(rnd: isize);
    async fn sum(sum: isize);
}

impl AdditiveSecretSharing for Party {
    type ShareFut = Ready<isize>;
    type SumFut   = Ready<isize>;

    fn start(self, _: context::Context, id



    fn share(self, _: context::Context, id: usize) -> Self::ShareFut {
        info!("party {id} started");
        let split = split(salaries[id]).await;

        info!("party {id} calculated split {split}.");

        let port = 12121 + id;
        let socket = format!("localhost:{port}");
        let listener = TcpListener::bind(socket);

        info!("party {id} computed average salary of {avg_sallary}.");
        // this is better/worse
        info!("party terminating ..");
    }

    fn sum(self, _: context:Context, id: usize) -> Self::SumFut {
        info!("party {id} started");


        future::ready(sum);
    }


}

#[tokio::main]
async fn main() {
    info!("main");
    const parties: = 0..parties_count;
    for id in 0..parties {
        let handle = tokio::spawn(party(id));
    }

    for id in parties {
        // join
    }

    println!("Hello, world!")

}

async fn split(salary: isize) -> [isize; parties_count] {
    let mut split: [isize; parties_count] = [0; parties_count];

    let mut acc=0;
    for id in 0..parties_count-1 {
        split[id] = rand::random::<isize>();
        acc += split[id];
    }

    *split.last_mut().unwrap() = salary - acc;

    split
}


