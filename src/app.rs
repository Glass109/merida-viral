use topcoat::router::{BodyLimit, Router, RouterBuilderDirectoryExt, RouterBuilderDiscoverExt};
pub(crate) fn build_application_router() -> Router {
    Router::builder()
        .layer(BodyLimit::max(2 * 1024 * 1024 + 65536).at("/images"))
        .discover()
        .serve_dir("/{*file}", "public")
        .build()
}
