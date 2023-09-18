use std::sync::Arc;

use futures::future::join_all;
use log::{error, info};

use fetcher::fetch_auctions;

use crate::prop_house::cacher::{
    get_auction_cache, get_proposal_cache, get_vote_cache, set_auctions_cache, set_proposals_cache,
    set_votes_cache,
};
use crate::prop_house::fetcher::{fetch_proposals, fetch_votes};
use crate::prop_house::handler::{handle_new_auction, handle_new_proposal, handle_new_vote};

mod cacher;
mod fetcher;
mod handler;

pub async fn setup() {
    if let Some(auctions) = fetch_auctions().await {
        set_auctions_cache(&auctions).unwrap();
    }

    if let Some(proposals) = fetch_proposals().await {
        set_proposals_cache(&proposals).unwrap();
    }

    if let Some(votes) = fetch_votes().await {
        set_votes_cache(&votes).unwrap();
    }
}

pub async fn start() {
    if let Some(auctions) = fetch_auctions().await {
        let mut tasks = Vec::new();

        for auction in auctions {
            let arc_auction = Arc::new(auction);
            if let Ok(cached_auction) = get_auction_cache(arc_auction.id.try_into().unwrap()) {
                let task = tokio::spawn({
                    let arc_auction = Arc::clone(&arc_auction);
                    async move {
                        if cached_auction.is_none() {
                            info!("Handle a new auction... ({:?})", arc_auction.id);
                            let _ = handle_new_auction(&arc_auction)
                                .await
                                .map_err(|err| error!("Failed to handle new auction: {:?}", err));
                        }
                    }
                });

                tasks.push(task);
            }
        }

        join_all(tasks).await;
    }
    if let Some(proposals) = fetch_proposals().await {
        let mut tasks = Vec::new();

        for proposal in proposals {
            let arc_proposal = Arc::new(proposal);
            if let Ok(cached_proposal) = get_proposal_cache(arc_proposal.id.try_into().unwrap()) {
                let task = tokio::spawn({
                    let arc_proposal = Arc::clone(&arc_proposal);
                    async move {
                        if cached_proposal.is_none() {
                            info!("Handle a new proposal... ({:?})", arc_proposal.id);
                            let _ = handle_new_proposal(&arc_proposal)
                                .await
                                .map_err(|err| error!("Failed to handle new proposal: {:?}", err));
                        }
                    }
                });

                tasks.push(task);
            }
        }

        join_all(tasks).await;
    }
    if let Some(votes) = fetch_votes().await {
        let mut tasks = Vec::new();

        for vote in votes {
            let arc_vote = Arc::new(vote);
            if let Ok(cached_vote) = get_vote_cache(arc_vote.id.try_into().unwrap()) {
                let task = tokio::spawn({
                    let arc_vote = Arc::clone(&arc_vote);
                    async move {
                        if cached_vote.is_none() {
                            info!("Handle a new vote... ({:?})", arc_vote.id);
                            let _ = handle_new_vote(&arc_vote)
                                .await
                                .map_err(|err| error!("Failed to handle new vote: {:?}", err));
                        }
                    }
                });

                tasks.push(task);
            }
        }

        join_all(tasks).await;
    }
}
