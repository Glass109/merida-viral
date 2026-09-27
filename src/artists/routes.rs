use chrono::NaiveDate;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::{Form, Html},
        error::{RouterErrorExt, SeeOther, see_other},
        path_param, route,
    },
};

use super::{BandAssignment, NewArtistForm, NewAssignmentForm, repo, views};
use crate::{bands, i18n::Text, images};

path_param!(band_id: i64);
path_param!(artist_id: i64);

#[route(GET "/artists")]
async fn list_artists(cx: &Cx) -> Result<Html<String>> {
    let artists = repo::list().await?;
    Ok(Html(views::list(&Text::new(cx), &artists).into_string()))
}

#[route(GET "/artists/{artist_id}")]
async fn artist_detail(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<ArtistId>(cx).ok_or_not_found()?;
    let artist = repo::find(id).await?.ok_or_not_found()?;
    let bands = repo::bands_for_artist(id).await?;
    Ok(Html(
        views::detail(&Text::new(cx), &artist, &bands).into_string(),
    ))
}

#[route(GET "/artists/{artist_id}/edit")]
async fn edit_artist(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<ArtistId>(cx).ok_or_not_found()?;
    let artist = repo::find(id).await?.ok_or_not_found()?;
    Ok(Html(views::edit(&Text::new(cx), &artist).into_string()))
}

#[route(POST "/artists/{artist_id}/edit")]
async fn update_artist(cx: &Cx, Form(input): Form<NewArtistForm>) -> Result<SeeOther> {
    let id = *path_param::<ArtistId>(cx).ok_or_not_found()?;
    let name = input.name.trim();
    if name.is_empty()
        || name.chars().count() > 90
        || !images::valid_image_url_for("artist", &input.image_url)
    {
        return Err(
            topcoat::router::error::bad_request(Text::new(cx).translate("invalid-artist")).into(),
        );
    }
    if !repo::update(
        id,
        name.to_owned(),
        images::normalized_image_url(input.image_url),
    )
    .await?
    {
        return Err(topcoat::router::error::not_found().into());
    }
    Ok(see_other(format!("/artists/{id}")))
}

#[route(GET "/artists/new")]
async fn new_artist(cx: &Cx) -> Result<Html<String>> {
    Ok(Html(views::new(&Text::new(cx)).into_string()))
}

#[route(POST "/artists/new")]
async fn create_artist(cx: &Cx, Form(input): Form<NewArtistForm>) -> Result<SeeOther> {
    let name = input.name.trim();
    if name.is_empty()
        || name.chars().count() > 90
        || !images::valid_image_url_for("artist", &input.image_url)
    {
        return Err(
            topcoat::router::error::bad_request(Text::new(cx).translate("invalid-artist")).into(),
        );
    }
    repo::create(
        name.to_owned(),
        images::normalized_image_url(input.image_url),
    )
    .await?;
    Ok(see_other("/artists"))
}

#[route(GET "/bands/{band_id}/artists/new")]
async fn new_assignment(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<BandId>(cx).ok_or_not_found()?;
    let band = bands::repo::find(id).await?.ok_or_not_found()?;
    let artists = repo::list().await?;
    Ok(Html(
        views::new_assignment(&Text::new(cx), &band, &artists).into_string(),
    ))
}

#[route(POST "/bands/{band_id}/artists/new")]
async fn create_assignment(cx: &Cx, Form(input): Form<NewAssignmentForm>) -> Result<SeeOther> {
    let band_id = *path_param::<BandId>(cx).ok_or_not_found()?;
    let date = |value: Option<String>| -> Option<Option<NaiveDate>> {
        match value.as_deref().map(str::trim) {
            None | Some("") => Some(None),
            Some(raw) => NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok().map(Some),
        }
    };
    let (Some(started_at), Some(ended_at)) = (date(input.started_at), date(input.ended_at)) else {
        return Err(topcoat::router::error::bad_request(
            Text::new(cx).translate("invalid-assignment"),
        )
        .into());
    };
    if input.artist_id <= 0
        || input.role.chars().count() > 90
        || started_at
            .zip(ended_at)
            .is_some_and(|(start, end)| start > end)
    {
        return Err(topcoat::router::error::bad_request(
            Text::new(cx).translate("invalid-assignment"),
        )
        .into());
    }
    if !repo::assign(BandAssignment {
        id: 0,
        band_id,
        artist_id: input.artist_id,
        role: input.role.trim().to_owned(),
        started_at,
        ended_at,
    })
    .await?
    {
        return Err(topcoat::router::error::bad_request(
            Text::new(cx).translate("invalid-assignment"),
        )
        .into());
    }
    Ok(see_other(format!("/bands/{band_id}")))
}
