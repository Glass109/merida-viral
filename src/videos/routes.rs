use crate::{
    geo,
    i18n::Text,
    videos::{
        embed::{self, EmbedData},
        model::{EmbedRequest, NewVideo, PlaceQuery, VideoForm, VoteInput},
        repo, thumbs, url, views,
    },
};
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::{Form, Html, Json},
        error::RouterErrorExt,
        path_param, query_params, route,
    },
};
path_param!(video_id: i64);

#[route(GET "/places")]
async fn list_places(cx: &Cx) -> Result<Html<String>> {
    let sort = query_params::<PlaceQuery>(cx)?;
    let newest = sort.sort.as_deref() == Some("newest");
    let videos = repo::list_videos(newest).await?;
    Ok(views::places_list_fragment(&Text::new(cx), &videos))
}
#[route(POST "/vote")]
async fn vote_video(cx: &Cx, Form(input): Form<VoteInput>) -> Result<Html<String>> {
    let sort = query_params::<PlaceQuery>(cx)?;
    let newest = sort.sort.as_deref() == Some("newest");
    repo::increment_vote(input.id).await?;
    let videos = repo::list_videos(newest).await?;
    Ok(views::places_list_fragment(&Text::new(cx), &videos))
}
#[route(POST "/video-vote")]
async fn vote_on_video_detail(cx: &Cx, Form(input): Form<VoteInput>) -> Result<Html<String>> {
    let votes = repo::increment_vote(input.id).await?;
    Ok(Html(
        views::video_detail_vote_button(&Text::new(cx), input.id, votes).into_string(),
    ))
}

/// Read the pasted link so the form can prefill the title and show a preview.
#[route(POST "/embed")]
async fn preview_video_embed(Json(input): Json<EmbedRequest>) -> Result<Json<EmbedData>> {
    let url = url::clean_video_url(&input.url);
    if !url.starts_with("https://") {
        return Err(topcoat::router::error::bad_request("invalid url").into());
    }
    Ok(Json(embed::lookup_video_embed(&url).await))
}

#[route(POST "/videos")]
async fn create_video(cx: &Cx, Form(input): Form<VideoForm>) -> Result<Html<String>> {
    let url = url::clean_video_url(&input.url);
    if !url.starts_with("https://") || !geo::valid_coordinates(input.lat, input.lng) {
        return Err(invalid_video_submission(cx));
    }

    let fetched = embed::lookup_video_embed(&url).await;
    let title = match input.title.trim() {
        "" => clamp_video_title(&fetched.title),
        _ => clamp_video_title(&input.title),
    };
    if title.is_empty() {
        return Err(invalid_video_submission(cx));
    }

    let neighborhood = geo::neighborhood_for_coordinates(input.lat, input.lng).await;
    let id = repo::create_video(&NewVideo {
        title,
        creator: fetched.creator,
        platform: fetched.platform,
        url,
        lat: input.lat,
        lng: input.lng,
        neighborhood,
        thumbnail: fetched.thumbnail.clone(),
    })
    .await?;
    if let Some(local) = thumbs::download_thumbnail_to_local_storage(id, &fetched.thumbnail).await {
        repo::set_thumbnail(id, &local).await?;
    }
    // Built after the awaits so the non-`Send` localisation isn't held across them.
    let text = Text::new(cx);
    Ok(Html(maud::html!{ p class="success-message" role="status" { (text.translate("added-success")) " " a href="/" { (text.translate("explore-merida")) } } }.into_string()))
}

fn invalid_video_submission(cx: &Cx) -> topcoat::Error {
    topcoat::router::error::bad_request(Text::new(cx).translate("invalid-video")).into()
}

/// Keep titles within the length the form allows, adding an ellipsis if cut.
fn clamp_video_title(title: &str) -> String {
    let title = title.trim();
    let mut chars = title.chars();
    let mut out: String = chars.by_ref().take(90).collect();
    if chars.next().is_some() {
        out.pop();
        out.push('…');
    }
    out
}
#[route(GET "/videos")]
async fn show_home_page(cx: &Cx) -> Result<Html<String>> {
    Ok(views::home_page(&Text::new(cx)))
}
#[route(GET "/submit")]
async fn show_video_submission_page(cx: &Cx) -> Result<Html<String>> {
    Ok(views::video_submission_page(&Text::new(cx)))
}
#[route(GET "/video/{video_id}")]
async fn show_video_detail_page(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<VideoId>(cx).ok_or_not_found()?;
    let video = repo::find_video(id).await?.ok_or_not_found()?;
    Ok(views::video_detail_page(&Text::new(cx), &video))
}
