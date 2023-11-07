use axum::{Router};
use axum::routing::get;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Pid of podnum instance
    #[arg(short, long)]
    pid: u64,
    // Node pids
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    nodes: Vec<u64>,
}

struct AppState {

}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Pid {}!", args.pid);
    println!("nodes {:?}", args.nodes);

    let app = Router::new()
        .route("/", get(|| async {
            "Hello, world!"
        }));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
