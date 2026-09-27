use topcoat::{
    Result,
    context::Cx,
    router::{
        content::{Form, Html},
        error::{RouterErrorExt, SeeOther, see_other},
        path_param, route,
    },
};

use super::{NewVenueForm, Venue, repo, views};
use crate::{geo, i18n::Text, images};

path_param!(venue_id: i64);

#[route(GET "/venues")]
async fn list_venues(cx: &Cx) -> Result<Html<String>> {
    let venues = repo::list().await?;
    Ok(Html(views::list(&Text::new(cx), &venues).into_string()))
}

#[route(GET "/venues/{venue_id}")]
async fn venue_detail(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<VenueId>(cx).ok_or_not_found()?;
    let venue = repo::find(id).await?.ok_or_not_found()?;
    Ok(Html(views::detail(&Text::new(cx), &venue).into_string()))
}

#[route(GET "/venues/{venue_id}/edit")]
async fn edit_venue(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<VenueId>(cx).ok_or_not_found()?;
    let venue = repo::find(id).await?.ok_or_not_found()?;
    Ok(Html(views::edit(&Text::new(cx), &venue).into_string()))
}

#[route(POST "/venues/{venue_id}/edit")]
async fn update_venue(cx: &Cx, Form(input): Form<NewVenueForm>) -> Result<SeeOther> {
    let id = *path_param::<VenueId>(cx).ok_or_not_found()?;
    let name = input.name.trim();
    let address = input.address.trim();
    if name.is_empty()
        || name.chars().count() > 90
        || address.is_empty()
        || address.chars().count() > 200
        || !geo::valid_coordinates(input.lat, input.lng)
        || !images::valid_image_url_for("venue", &input.image_url)
    {
        return Err(
            topcoat::router::error::bad_request(Text::new(cx).translate("invalid-venue")).into(),
        );
    }
    let venue = Venue {
        id,
        name: name.to_owned(),
        address: address.to_owned(),
        location: (input.lng, input.lat),
        tags: input.tags(),
        image_url: images::normalized_image_url(input.image_url),
    };
    if !repo::update(venue).await? {
        return Err(topcoat::router::error::not_found().into());
    }
    Ok(see_other(format!("/venues/{id}")))
}

#[route(GET "/venues/new")]
async fn new_venue(cx: &Cx) -> Result<Html<String>> {
    Ok(Html(views::new(&Text::new(cx)).into_string()))
}

#[route(POST "/venues/new")]
async fn create_venue(cx: &Cx, Form(input): Form<NewVenueForm>) -> Result<SeeOther> {
    let name = input.name.trim();
    let address = input.address.trim();
    if name.is_empty()
        || name.chars().count() > 90
        || address.is_empty()
        || address.chars().count() > 200
        || !geo::valid_coordinates(input.lat, input.lng)
        || !images::valid_image_url_for("venue", &input.image_url)
    {
        return Err(
            topcoat::router::error::bad_request(Text::new(cx).translate("invalid-venue")).into(),
        );
    }
    let venue = Venue {
        id: 0,
        name: name.to_owned(),
        address: address.to_owned(),
        location: (input.lng, input.lat),
        tags: input.tags(),
        image_url: images::normalized_image_url(input.image_url),
    };
    let id = repo::create(venue).await?;
    Ok(see_other(format!("/venues/{id}")))
}
