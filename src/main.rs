mod playlist;
use playlist::get_playlist_id_create_if_needed;

mod spotify;
use spotify::SpotifyAPI;

extern crate rspotify;
use rspotify::spotify::oauth2::SpotifyOAuth;

extern crate argparse;
use argparse::{ArgumentParser, Store};

fn main() {
    // Parse arguments
    let mut client_id = String::new();
    let mut client_secret = String::new();
    let mut username = String::new();
    let mut playlist_name = String::new();
    {
        // Create parser in scope so that we can retrieve borrowed values
        // after parser is released
        let mut parser = ArgumentParser::new();
        parser.set_description("Create a playlist with songs from a csv");
        parser.refer(&mut client_id)
            .add_argument("client_id", Store,
                          "Spotify Client ID")
            .required();
        parser.refer(&mut client_secret)
            .add_argument("client_secret", Store,
                          "Spotify Client Secret")
            .required();
        parser.refer(&mut username)
            .add_argument("username", Store,
                          "Spotify Username")
            .required();
        parser.refer(&mut playlist_name)
            .add_argument("playlist_name", Store,
                          "Spotify Playlist name")
            .required();
        parser.parse_args_or_exit();
    }
    // Set up Spotify OAuth
    let mut oauth = SpotifyOAuth::default()
        .scope("playlist-modify-private")
        .client_id(&client_id)
        .client_secret(&client_secret)
        .redirect_uri("http://localhost:8888/callback")
        .build();
    // Log in with username
    let spotify = SpotifyAPI::new(&username, &mut oauth).unwrap();
    // Get playlist ID from playlist name
    let playlist_id = get_playlist_id_create_if_needed(&spotify, &playlist_name).unwrap();
}
