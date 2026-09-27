use native_db::db_type::Error;

use crate::db::{db, invalid_data, next_video_id, run_db};

use super::model::{NewVideo, Video};

/// Newest first, or most votes first with newest as the tie breaker.
pub(crate) async fn list_videos(newest: bool) -> Result<Vec<Video>, Error> {
    run_db(move || {
        let txn = db().r_transaction()?;
        let mut videos: Vec<Video> = txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        if newest {
            videos.sort_unstable_by_key(|video| std::cmp::Reverse(video.id));
        } else {
            videos.sort_unstable_by_key(|video| {
                (std::cmp::Reverse(video.votes), std::cmp::Reverse(video.id))
            });
        }
        Ok(videos)
    })
    .await
}

pub(crate) async fn find_video(id: i64) -> Result<Option<Video>, Error> {
    run_db(move || db().r_transaction()?.get().primary::<Video>(id)).await
}

/// Read and update in one write transaction so concurrent votes cannot be lost.
pub(crate) async fn increment_vote(id: i64) -> Result<i64, Error> {
    run_db(move || {
        let txn = db().rw_transaction()?;
        let old: Video = txn
            .get()
            .primary(id)?
            .ok_or_else(|| invalid_data(format!("video {id} not found")))?;
        let mut video = old.clone();
        video.votes = video
            .votes
            .checked_add(1)
            .ok_or_else(|| invalid_data("vote count overflow"))?;
        let votes = video.votes;
        txn.update(old, video)?;
        txn.commit()?;
        Ok(votes)
    })
    .await
}

pub(crate) async fn create_video(input: &NewVideo) -> Result<i64, Error> {
    let video = Video {
        id: 0,
        title: input.title.trim().to_owned(),
        creator: input.creator.trim().to_owned(),
        platform: input.platform.trim().to_owned(),
        url: input.url.trim().to_owned(),
        lat: input.lat,
        lng: input.lng,
        neighborhood: input.neighborhood.trim().to_owned(),
        thumbnail: input.thumbnail.trim().to_owned(),
        votes: 0,
    };
    run_db(move || {
        let id = next_video_id()?;
        let txn = db().rw_transaction()?;
        txn.insert(Video { id, ..video })?;
        txn.commit()?;
        Ok(id)
    })
    .await
}

pub(crate) async fn set_thumbnail(id: i64, thumbnail: &str) -> Result<(), Error> {
    let thumbnail = thumbnail.to_owned();
    run_db(move || {
        let txn = db().rw_transaction()?;
        if let Some(old) = txn.get().primary::<Video>(id)? {
            let mut video = old.clone();
            video.thumbnail = thumbnail;
            txn.update(old, video)?;
        }
        txn.commit()?;
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn videos_round_trip_and_votes_are_atomic() {
        crate::db::install_test_database();
        let input = NewVideo {
            title: " First ".into(),
            creator: " Creator ".into(),
            platform: "example".into(),
            url: "https://example.com".into(),
            lat: 20.0,
            lng: -89.0,
            neighborhood: " Centro ".into(),
            thumbnail: "".into(),
        };
        let first = create_video(&input).await.unwrap();
        let second = create_video(&input).await.unwrap();
        assert_eq!((first, second), (1, 2));
        assert_eq!(find_video(first).await.unwrap().unwrap().title, "First");
        assert!(find_video(999).await.unwrap().is_none());

        let votes = (0..16).map(|_| tokio::spawn(increment_vote(first)));
        for vote in votes {
            vote.await.unwrap().unwrap();
        }
        assert_eq!(find_video(first).await.unwrap().unwrap().votes, 16);
        increment_vote(second).await.unwrap();
        assert_eq!(
            list_videos(true)
                .await
                .unwrap()
                .iter()
                .map(|video| video.id)
                .collect::<Vec<_>>(),
            vec![2, 1]
        );
        assert_eq!(
            list_videos(false)
                .await
                .unwrap()
                .iter()
                .map(|video| video.id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );

        set_thumbnail(first, "local.jpg").await.unwrap();
        assert_eq!(
            find_video(first).await.unwrap().unwrap().thumbnail,
            "local.jpg"
        );
        assert!(increment_vote(999).await.is_err());
    }
}
