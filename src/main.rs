mod playlist;
use playlist::get_playlist_id_create_if_needed;

mod spotify;
use spotify::SpotifyAPI;

extern crate rspotify;
use rspotify::spotify::oauth2::SpotifyOAuth;

fn main() {
    // Set up Spotify OAuth
    let mut oauth = SpotifyOAuth::default()
        .scope("playlist-modify-private")
        .client_id("<ENTER-YOUR-VALUES-HERE>")
        .client_secret("<ENTER-YOUR-VALUES-HERE>")
        .redirect_uri("http://localhost:8888/callback")
        .build();
    let username = "<ENTER-USERNAME-HERE>";
    let playlist_name = "wedding";
    // Log in with username
    let spotify = SpotifyAPI::new(username, &mut oauth).unwrap();
    // Get playlist ID from playlist name
    let playlist_id = get_playlist_id_create_if_needed(&spotify, playlist_name).unwrap();
}
