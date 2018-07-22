use std::error::Error;
use std::fmt;

use playlist::{PlaylistAPI,PlaylistError,PlaylistNotFound};

extern crate rspotify;

use rspotify::spotify::client::Spotify;
use rspotify::spotify::util::get_token;
use rspotify::spotify::oauth2::{SpotifyClientCredentials,SpotifyOAuth};

extern crate failure;

/// An error when authentication fails to Spotify servers
#[derive(Debug)]
pub struct AuthenticationFailed;

impl Error for AuthenticationFailed {
    fn description(&self) -> &str {
        "Authentication failed"
    }
}

impl fmt::Display for AuthenticationFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Stores necessary information for calling Spotify API
pub struct SpotifyAPI {
    spotify: Spotify,
    username: String,
}

impl SpotifyAPI {
    /// Returns a SpotifyAPI that will query using the given username and auth
    ///
    /// # Arguments
    ///
    /// * `username` - A string slice that holds the username
    /// * `spotify_oauth` - A setup OAuth struct
    pub fn new(username: &str, mut spotify_oauth: &mut SpotifyOAuth) -> Result<SpotifyAPI, AuthenticationFailed> {
        match get_token(&mut spotify_oauth) {
            Some(token_info) => {
                let client_credential = SpotifyClientCredentials::default()
                    .token_info(token_info)
                    .build();
                let spotify = Spotify::default()
                    .client_credentials_manager(client_credential)
                    .build();
                Ok(SpotifyAPI{spotify: spotify, username: username.to_owned()})
            }
            None => Err(AuthenticationFailed{}),
        }
    }
}

impl PlaylistAPI<failure::Error> for SpotifyAPI {
    fn get_playlist_id(&self, playlist_name: &str) -> Result<String, PlaylistError<failure::Error>> {
        let playlist_page = self.spotify.current_user_playlists(None, None).map_err(PlaylistError::APIError)?;
        // Find the first playlist with the matching name
        for p in playlist_page.items {
            if p.name == playlist_name {
                return Ok(p.id);
            };
        };
        // Send error if we don't find the playlist
        Err(PlaylistError::PlaylistNotFound(PlaylistNotFound{}))
    }

    fn create_playlist(&self, playlist_name: &str) -> Result<String, failure::Error> {
        let playlist = self.spotify.user_playlist_create(&self.username, playlist_name, false, None)?;
        Ok(playlist.id)
    }
}
