use std::error::Error;
use std::fmt;

use playlist::{PlaylistAPI,PlaylistError,PlaylistNotFound};

extern crate rspotify;

use rspotify::spotify::client::Spotify;
use rspotify::spotify::util::get_token;
use rspotify::spotify::oauth2::{SpotifyClientCredentials,SpotifyOAuth};
use rspotify::spotify::model::playlist::PlaylistTrack;

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
    pub fn new(username: &str,
               mut spotify_oauth: &mut SpotifyOAuth) -> Result<SpotifyAPI, AuthenticationFailed> {
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

impl SpotifyAPI {
    /// Get playlist ID by searching through pages of playlists.
    /// This will be recursively called incrementing offset.
    fn get_playlist_id_with_offset(&self,
                                   playlist_name: &str,
                                   offset: u32) -> Result<String, PlaylistError<failure::Error>> {
        let result = self.spotify.current_user_playlists(None, Some(offset));
        let playlist_page = result.map_err(PlaylistError::APIError)?;
        // Find the first playlist with the matching name
        for p in playlist_page.items {
            if p.name == playlist_name {
                return Ok(p.id.to_owned());
            };
        };
        if playlist_page.total < playlist_page.limit {
            // Send error if we don't find the playlist
            return Err(PlaylistError::PlaylistNotFound(PlaylistNotFound{}));
        }
        // Recurse over the next page
        self.get_playlist_id_with_offset(playlist_name, offset + playlist_page.total)
    }
}

impl PlaylistAPI<failure::Error> for SpotifyAPI {
    fn get_playlist_id(&self,
                       playlist_name: &str) -> Result<String, PlaylistError<failure::Error>> {
        self.get_playlist_id_with_offset(playlist_name, 0)
    }

    fn create_playlist(&self,
                       playlist_name: &str) -> Result<String, failure::Error> {
        let playlist = self.spotify.user_playlist_create(
            &self.username,
            playlist_name,
            false,
            None
        )?;
        Ok(playlist.id)
    }

    fn add_tracks_to_playlist(&self,
                              playlist_id: &str,
                              track_ids: &[String]) -> Result<(), failure::Error> {
        if track_ids.is_empty() {
            return Ok(());
        }
        self.spotify.user_playlist_add_tracks(
            &self.username,
            playlist_id,
            &track_ids,
            None
        )?;
        Ok(())
    }

    fn get_track_ids_in_playlist(&self,
                                 playlist_id: &str) -> Result<Vec<String>, failure::Error> {
        let results = self.spotify.user_playlist_tracks(
            &self.username,
            playlist_id,
            None,
            None,
            None,
            None
        )?;
        Ok(get_track_ids(results.items))
    }
}

/// Converts playlist track into just the IDs
fn get_track_ids(result: Vec<PlaylistTrack>) -> Vec<String> {
    result.iter().map(|x| x.clone().track.id).collect()
}
