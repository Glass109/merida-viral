use topcoat::{
    Result,
    context::Cx,
    router::{
        content::{Form, Html},
        error::{RouterErrorExt, SeeOther, see_other},
        path_param, route,
    },
};

use super::{NewBandForm, repo, views};
use crate::{artists, i18n::Text, images};

path_param!(band_id: i64);

#[route(GET "/bands/{band_id}")]
async fn band_detail(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<BandId>(cx).ok_or_not_found()?;
    let band = repo::find(id).await?.ok_or_not_found()?;
    let assignments = artists::repo::for_band(id).await?;
    Ok(Html(
        views::detail(&Text::new(cx), &band, &assignments).into_string(),
    ))
}

#[route(GET "/bands/{band_id}/edit")]
async fn edit_band(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<BandId>(cx).ok_or_not_found()?;
    let band = repo::find(id).await?.ok_or_not_found()?;
    Ok(Html(views::edit(&Text::new(cx), &band).into_string()))
}

#[route(POST "/bands/{band_id}/edit")]
async fn update_band(cx: &Cx, Form(input): Form<NewBandForm>) -> Result<SeeOther> {
    let id = *path_param::<BandId>(cx).ok_or_not_found()?;
    let name = input.name.trim();
    if name.is_empty()
        || name.chars().count() > 90
        || !images::valid_image_url_for("band", &input.image_url)
    {
        return Err(
            topcoat::router::error::bad_request(Text::new(cx).translate("invalid-band")).into(),
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
    Ok(see_other(format!("/bands/{id}")))
}

#[route(GET "/bands")]
async fn list_bands(cx: &Cx) -> Result<Html<String>> {
    let bands = repo::list().await?;
    Ok(Html(views::list(&Text::new(cx), &bands).into_string()))
}

#[route(GET "/bands/new")]
async fn new_band(cx: &Cx) -> Result<Html<String>> {
    Ok(Html(views::new(&Text::new(cx)).into_string()))
}

#[route(POST "/bands/new")]
async fn create_band(cx: &Cx, Form(input): Form<NewBandForm>) -> Result<SeeOther> {
    let name = input.name.trim();
    if name.is_empty()
        || name.chars().count() > 90
        || !images::valid_image_url_for("band", &input.image_url)
    {
        return Err(
            topcoat::router::error::bad_request(Text::new(cx).translate("invalid-band")).into(),
        );
    }
    repo::create(
        name.to_owned(),
        images::normalized_image_url(input.image_url),
    )
    .await?;
    Ok(see_other("/bands"))
}
