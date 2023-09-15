use std::sync::Arc;

use futures::future::join_all;
use log::{error, info};

pub use fetcher::fetch_auctions;

use crate::prop_house::cacher::{get_auction_cache, set_auction_cache};
use crate::prop_house::handler::handle_new_auction;

mod cacher;
mod fetcher;
mod handler;

pub async fn setup() {
    let auctions = fetch_auctions().await;

    if let Some(auction_list) = auctions {
        let mut tasks = Vec::new();

        for auction in auction_list {
            let arc_auction = Arc::new(auction);
            let task = tokio::spawn({
                let arc_auction = Arc::clone(&arc_auction);
                async move {
                    info!("Cache a new auction... ({:?})", arc_auction.id);
                    let _ = set_auction_cache(&*arc_auction).await.map_err(|e| {
                        error!("Error while trying to set auction cache: {}", e);
                    });
                }
            });

            tasks.push(task);
        }

        join_all(tasks).await;
    }
}

pub async fn start() {
    let auctions = fetch_auctions().await;

    if let Some(auction_list) = auctions {
        let mut tasks = Vec::new();

        for auction in auction_list {
            let arc_auction = Arc::new(auction);
            let cached_auction = get_auction_cache(arc_auction.id as i32).await;
            let task = tokio::spawn({
                let arc_auction = Arc::clone(&arc_auction);
                async move {
                    if cached_auction.is_none() {
                        info!("Handle a new auction... ({:?})", arc_auction.id);
                        let _ = handle_new_auction(&*arc_auction)
                            .await
                            .map_err(|err| error!("Failed to handle new auction: {:?}", err));
                    }
                }
            });

            tasks.push(task);
        }

        join_all(tasks).await;
    }
}
