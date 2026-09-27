# Mérida Underground

The home page (`/`) showcases upcoming events on a map; `/events` includes past shows and `/videos` retains the video explorer. Event creation offers eight built-in photo-mask map pins with an image preview. Pin selections are stored separately from existing event records; older events use the guitar-pick style. Pins use the event picture, fall back to the venue picture, then to a music symbol. Events sharing coordinates are cycled by selecting their shared marker, or located individually from the list.

Pin silhouettes live in `public/pin-masks/*.svg` (one editable SVG per shape, `viewBox="0 0 100 118"`). Draw opaque white paths on a transparent background; transparent areas become holes in the mask. `public/app.css` applies the same file to three layers: a black outer contour, a blurred and darkened copy of the picture with `public/noise.jpg` texture, and the sharp inset picture. Pins without a picture use the noise texture as a fallback. No SVG markup is inlined in event HTML. Adjust `.photo-pin-rim` and `.photo-pin-fill` inset values to change the outlines' thickness.

Run the Rust server with `cargo run`. The app stores records in `merida-underground.native_db` by default; set `DATABASE_PATH` to use a different file.

## Pictures

Artist, band, venue and event forms accept optional JPEG, PNG or WebP images up to 2 MiB. Files go into the configured **private** R2 bucket; the app serves them from `/images/{artist|band|venue|event}/{id}.{extension}`. R2 is not publicly exposed, and the browser does not need S3 credentials.

Events reference an existing venue and an ordered lineup of one or more existing bands. Enter start/end times in Mérida local time; records store UTC timestamps. An optional end time must be after the start time.

Configure the Rust server with `R2_ENDPOINT`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, and `R2_BUCKET`. The R2 S3 token must have **Object Read & Write** on the configured bucket; Wrangler login alone does not give the running server S3 access. For local development the app also reads the existing, gitignored `config.toml` key/value file (`S3_ENDPOINT`, `S3_ACCESS_KEY_ID`, `s3_SECRET_ACCESS_KEY`, `S3_BUCKET_NAME`) if environment variables are absent. This is not standard TOML. Bucket configuration is loaded on the first image request and cached until server restart. Never commit credentials.

**Security:** Until admin middleware is installed, the create and upload routes are public. The server validates image type and size, but it does not authenticate callers or rate-limit uploads. Do not expose this deployment publicly without protecting those routes.
