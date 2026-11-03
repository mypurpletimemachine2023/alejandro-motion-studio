//! Community and project links (shown in Help ▸, the About dialog, the start screen and the
//! header's Discord button).

pub const APP_NAME: &str = "ACA Motion Lab";
pub const APP_SLUG: &str = "effectcraft";
pub const DISCORD: &str = "/2d/motion#tutorial";
pub const WEBSITE: &str = "/2dhome";
pub const APP_PAGE: &str = "/2d/motion";
pub const GITHUB: &str = "/credits/";
pub const ISSUES: &str = "/credits/";

/// Sibling apps of the ArtCraft family: (name, what it is, GitHub repo, app page).
pub const SIBLINGS: &[(&str, &str)] =
    &[("ACA Photo Lab", "photocraft"), ("ACA Vector Lab", "vectorcraft"), ("ACA Video Lab", "filmcraft"), ("ACA Audio Lab", "soundcraft")];

pub fn github_of(slug: &str) -> String {
    let _ = slug;
    "/credits/".into()
}
pub fn page_of(slug: &str) -> String {
    match slug {
        "photocraft" => "/2d/photo",
        "vectorcraft" => "/2d/vector",
        "filmcraft" => "/2d/video",
        "soundcraft" => "/2d/audio",
        _ => "/2dhome",
    }
    .into()
}
