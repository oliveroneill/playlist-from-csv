#[macro_use]
extern crate serde_derive;

mod dynamodb;
use dynamodb::{parse_csv_file,add_songs_to_playlist};

mod playlist;
use playlist::{get_playlist_id_create_if_needed};

mod spotify;
use spotify::SpotifyAPI;

extern crate rspotify;
use rspotify::spotify::oauth2::SpotifyOAuth;

extern crate argparse;
use argparse::{ArgumentParser, Store};

fn update_playlist_from_csv(client_id: &str, client_secret: &str,
                            username: &str, playlist_name: &str,
                            csv_filename: &str) {
    // Set up Spotify OAuth
    let mut oauth = SpotifyOAuth::default()
        .scope("playlist-read-private playlist-modify-private")
        .client_id(&client_id)
        .client_secret(&client_secret)
        .redirect_uri("http://localhost:8888/callback")
        .build();
    // Log in with username
    let spotify = SpotifyAPI::new(&username, &mut oauth).unwrap();
    // Get playlist ID from playlist name
    let playlist_id = get_playlist_id_create_if_needed(&spotify, &playlist_name).unwrap();
    let songs = parse_csv_file(csv_filename).unwrap();
    add_songs_to_playlist(&spotify, &playlist_id, songs).unwrap();
    println!("Successfully added songs!");
}

fn main() {
    // Parse arguments
    let mut client_id = String::new();
    let mut client_secret = String::new();
    let mut username = String::new();
    let mut playlist_name = String::new();
    let mut csv_filename = String::new();
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
        parser.refer(&mut csv_filename)
            .add_argument("csv_filename", Store,
                          "CSV Filename")
            .required();
        parser.parse_args_or_exit();
    }
    update_playlist_from_csv(
        &client_id, &client_secret, &username, &playlist_name, &csv_filename
    );
}
