use crate::{
    i18n::Text,
    videos::{
        embed, geo,
        model::{EmbedPreview, EmbedRequest, NewVideo, PlaceQuery, VideoForm, VoteInput},
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
async fn places(cx: &Cx) -> Result<Html<String>> {
    let sort = query_params::<PlaceQuery>(cx)?;
    Ok(views::places_fragment(
        &Text::new(cx),
        sort.sort.as_deref() == Some("newest"),
    ))
}
#[route(POST "/vote")]
async fn vote_video(cx: &Cx, Form(input): Form<VoteInput>) -> Result<Html<String>> {
    let sort = query_params::<PlaceQuery>(cx)?;
    repo::increment_vote(input.id);
    Ok(views::places_fragment(
        &Text::new(cx),
        sort.sort.as_deref() == Some("newest"),
    ))
}
#[route(POST "/video-vote")]
async fn vote_detail(cx: &Cx, Form(input): Form<VoteInput>) -> Result<Html<String>> {
    let votes = repo::increment_vote(input.id);
    Ok(Html(
        views::detail_vote(&Text::new(cx), input.id, votes).into_string(),
    ))
}

/// Read the pasted link so the form can prefill the title and show a preview.
#[route(POST "/embed")]
async fn embed_preview(Json(input): Json<EmbedRequest>) -> Result<Json<EmbedPreview>> {
    let url = url::clean(&input.url);
    if !url.starts_with("https://") {
        return Err(topcoat::router::error::bad_request("invalid url").into());
    }
    let data = embed::lookup(&url).await;
    Ok(Json(EmbedPreview {
        title: data.title,
        creator: data.creator,
        thumbnail: data.thumbnail,
        platform: data.platform,
    }))
}

#[route(POST "/videos")]
async fn create_video(cx: &Cx, Form(input): Form<VideoForm>) -> Result<Html<String>> {
    let url = url::clean(&input.url);
    if !url.starts_with("https://")
        || !(-90.0..=90.0).contains(&input.lat)
        || !(-180.0..=180.0).contains(&input.lng)
    {
        return Err(invalid_video(cx));
    }

    let fetched = embed::lookup(&url).await;
    let title = match input.title.trim() {
        "" => clamp_title(&fetched.title),
        _ => clamp_title(&input.title),
    };
    if title.is_empty() {
        return Err(invalid_video(cx));
    }

    let neighborhood = geo::neighborhood(input.lat, input.lng).await;
    let id = repo::create_video(&NewVideo {
        title,
        creator: fetched.creator,
        platform: fetched.platform,
        url,
        lat: input.lat,
        lng: input.lng,
        neighborhood,
        thumbnail: fetched.thumbnail.clone(),
    });
    if let Some(local) = thumbs::cache(id, &fetched.thumbnail).await {
        repo::set_thumbnail(id, &local);
    }
    // Built after the awaits so the non-`Send` localisation isn't held across them.
    let text = Text::new(cx);
    Ok(Html(maud::html!{ p class="success-message" role="status" { (text.t("added-success")) " " a href="/" { (text.t("explore-merida")) } } }.into_string()))
}

fn invalid_video(cx: &Cx) -> topcoat::Error {
    topcoat::router::error::bad_request(Text::new(cx).t("invalid-video")).into()
}

/// Keep titles within the length the form allows, adding an ellipsis if cut.
fn clamp_title(title: &str) -> String {
    let title = title.trim();
    let mut chars = title.chars();
    let mut out: String = chars.by_ref().take(90).collect();
    if chars.next().is_some() {
        out.pop();
        out.push('…');
    }
    out
}
#[route(GET "/")]
async fn home(cx: &Cx) -> Result<Html<String>> {
    Ok(views::home_page(&Text::new(cx)))
}
#[route(GET "/submit")]
async fn submit(cx: &Cx) -> Result<Html<String>> {
    Ok(views::submit_page(&Text::new(cx)))
}
#[route(GET "/video/{video_id}")]
async fn video(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<VideoId>(cx).ok_or_not_found()?;
    let video = repo::find_video(id).ok_or_not_found()?;
    Ok(views::video_page(&Text::new(cx), &video))
}
