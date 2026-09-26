use crate::videos;
use topcoat::router::{Router, RouterBuilderDirectoryExt, RouterBuilderDiscoverExt};
pub(crate) fn router() -> Router {
    videos::repo::initialize();
    Router::builder()
        .discover()
        .serve_dir("/{*file}", "public")
        .build()
}
