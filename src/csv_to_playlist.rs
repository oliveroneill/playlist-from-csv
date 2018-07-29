use std::fmt;
use std::error::Error;
use std::fs::File;

use playlist::{PlaylistAPI};

extern crate csv;

/// A struct containing relevant spotify information for playlist tracks.
/// This is specifically used for a DynamoDB export to CSV
#[derive(Clone, Debug, Deserialize)]
pub struct Song {
    /// A human readable name of the song
    #[serde(rename = "music (S)")]
    pub music: String,
    /// A Spotify ID for the track
    #[serde(rename = "song_id (S)")]
    pub song_id: String,
}

/// Parse a CSV file to retrieve song information. The fields
/// it should have are "music (S)" and "song_id (S)" as described
/// in the struct above.
///
/// # Arguments
///
/// * `filename` - The path to the CSV file
pub fn parse_csv_file(filename: &str) -> Result<Vec<Song>, Box<Error>> {
    let file = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut results = Vec::new();
    for result in rdr.deserialize() {
        let record: Song = result?;
        results.push(record);
    }
    Ok(results)
}

/// Used to get the ID out of the Song struct
fn get_track_id_from_song(song: &Song) -> String {
    song.song_id.to_owned()
}

/// Playlist error enum for different errors when adding tracks to playlist
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlaylistAddError<E> {
    /// Generic playlist error from API
    APIError(E),
    /// The error when the playlist cannot be found
    NoNewTracks(NoNewTracks),
}

/// An error when there are no new tracks to add to the playlist
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NoNewTracks {}

impl Error for NoNewTracks {
    fn description(&self) -> &str {
        "No tracks to add to playlist"
    }
}

impl fmt::Display for NoNewTracks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Add songs to a playlist using the input API.
///
/// # Arguments
///
/// * `playlist_api` - The instance where the tracks should be added
/// * `playlist_id` - The playlist ID to be added to. This is the ID and *not*
///                   the name.
/// * `songs` - A vec of the songs
pub fn add_songs_to_playlist<E>(playlist_api: &PlaylistAPI<E>,
                                playlist_id: &str,
                                songs: Vec<Song>) -> Result<(), PlaylistAddError<E>> {
    // Map the songs to IDs
    let track_ids: Vec<String> = songs.iter().map(get_track_id_from_song).collect();
    let mut filtered = filter_duplicates(
        playlist_api, playlist_id, track_ids
    ).map_err(PlaylistAddError::APIError)?;
    // Sort so that dedup removes all duplicates
    filtered.sort();
    // Remove all duplicates
    filtered.dedup();
    // If there's no tracks left then send back a message to indicate that
    if filtered.is_empty() {
        return Err(PlaylistAddError::NoNewTracks(NoNewTracks {}));
    }
    // Add the IDs to the playlist
    playlist_api.add_tracks_to_playlist(
        playlist_id, &filtered[..]
    ).map_err(PlaylistAddError::APIError)?;
    Ok(())
}

/// Filter tracks that are already in the playlist.
///
/// # Arguments
///
/// * `playlist_api` - The instance where the tracks should be added
/// * `playlist_id` - The playlist ID to be added to. This is the ID and *not*
///                   the name.
/// * `track_ids` - A vec of IDs for each song
pub fn filter_duplicates<E>(playlist_api: &PlaylistAPI<E>,
                            playlist_id: &str,
                            track_ids: Vec<String>) -> Result<Vec<String>, E> {
    let tracks = playlist_api.get_track_ids_in_playlist(playlist_id)?;
    let filtered: Vec<String> = track_ids
        .iter()
        .filter(|id| !tracks.contains(id))
        .cloned()
        .collect();
    Ok(filtered)
}


#[cfg(test)]
mod tests {
    use super::*;
    use playlist::{PlaylistAPI,PlaylistError};

    use std::cell::RefCell;

    /// Keep track of calls made to MockPlaylistAPI
    #[derive(Debug, Clone)]
    struct CallHistory {
        create_playlist_called_with: Option<String>,
        get_playlist_id_called_with: Option<String>,
        add_tracks_to_playlist_called_with: Option<(String, Vec<String>)>,
        get_track_ids_in_playlist_called_with: Option<String>,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct FakeError {}

    /// A fake API where you can specify the return values to the functions
    struct MockPlaylistAPI {
        call_history: RefCell<CallHistory>,
        add_tracks_to_playlist_returns: Result<(), FakeError>,
        get_track_ids_in_playlist_returns: Result<Vec<String>, FakeError>,
    }

    impl MockPlaylistAPI {
        /// Create a new MockPlaylistAPI
        fn new(add_tracks_to_playlist_returns: Result<(), FakeError>,
               get_track_ids_in_playlist_returns: Result<Vec<String>, FakeError>) -> MockPlaylistAPI {
            MockPlaylistAPI {
                call_history: RefCell::new(
                    CallHistory{
                        create_playlist_called_with: None,
                        get_playlist_id_called_with: None,
                        add_tracks_to_playlist_called_with: None,
                        get_track_ids_in_playlist_called_with: None,
                    }
                ),
                add_tracks_to_playlist_returns: add_tracks_to_playlist_returns,
                get_track_ids_in_playlist_returns: get_track_ids_in_playlist_returns,
            }
        }
    }

    impl PlaylistAPI<FakeError> for MockPlaylistAPI {
        fn get_playlist_id(&self, playlist_name: &str) -> Result<String, PlaylistError<FakeError>> {
            let mut calls = self.call_history.borrow_mut();
            calls.get_playlist_id_called_with = Some(playlist_name.to_owned());
            Ok("".to_string())
        }

        fn create_playlist(&self, playlist_name: &str) -> Result<String, FakeError> {
            let mut calls = self.call_history.borrow_mut();
            calls.create_playlist_called_with = Some(playlist_name.to_owned());
            Ok("".to_string())
        }

        #[allow(unused_variables)]
        fn add_tracks_to_playlist(&self, playlist_id: &str, track_ids: &[String]) -> Result<(), FakeError> {
            let mut x = vec!["".to_string(); track_ids.len()];
            x.clone_from_slice(track_ids);
            let mut calls = self.call_history.borrow_mut();
            calls.add_tracks_to_playlist_called_with = Some((playlist_id.to_owned(), x));
            self.add_tracks_to_playlist_returns.clone()
        }

        #[allow(unused_variables)]
        fn get_track_ids_in_playlist(&self, playlist_id: &str) -> Result<Vec<String>, FakeError> {
            let mut calls = self.call_history.borrow_mut();
            calls.get_track_ids_in_playlist_called_with = Some(playlist_id.to_owned());
            self.get_track_ids_in_playlist_returns.clone()
        }
    }

    /// Used for declaring the inputs for each test
    fn test_setup() -> (String, [String; 3], Vec<Song>) {
        let playlist_name = "test_playlist_name1".to_string();
        // These must be in alphabetical order to make the tests simpler.
        // This is because add_songs_to_playlist performs a sort that changes
        // the order that tracks are added
        let expected_tracks = ["3ndjkfd9".to_string(), "asqww_nf".to_string(), "vvcs33".to_string()];
        // Create some songs to add
        let mut songs = Vec::new();
        songs.push(Song{music:"BLA".to_string(), song_id:expected_tracks[0].to_owned()});
        songs.push(Song{music:"test song".to_string(), song_id:expected_tracks[1].to_owned()});
        songs.push(Song{music:"another 1".to_string(), song_id:expected_tracks[2].to_owned()});
        (playlist_name, expected_tracks, songs)
    }

    #[test]
    fn add_songs_to_playlist_success() {
        // Given
        let (playlist_name, expected_tracks, songs) = test_setup();
        let api = MockPlaylistAPI::new(Ok(()), Ok(Vec::new()));
        // When
        // Ensure it doesn't fail using unwrap
        add_songs_to_playlist(&api, &playlist_name, songs).unwrap();
        // Then
        let expected = Some((playlist_name.to_string(), expected_tracks.to_vec()));
        let expected_track_id_call = Some(playlist_name.to_string());
        // Check the call history
        let calls = api.call_history.borrow();
        // Ensure that API was called correctly
        assert_eq!(expected, calls.add_tracks_to_playlist_called_with);
        assert_eq!(expected_track_id_call, calls.get_track_ids_in_playlist_called_with);
        // Ensure irrelevant function is not called
        assert_eq!(None, calls.create_playlist_called_with);
        assert_eq!(None, calls.get_playlist_id_called_with);
    }

    #[test]
    fn add_songs_to_playlist_error() {
        // Given
        let (playlist_name, expected_tracks, songs) = test_setup();
        let error = FakeError{};
        let api = MockPlaylistAPI::new(Err(error), Ok(Vec::new()));
        // When
        let result = add_songs_to_playlist(&api, &playlist_name, songs);
        // Then
        match result {
            // Ensure that we receive an error
            Ok(_) => assert!(false),
            // Make sure we receive the error
            Err(err) => assert_eq!(PlaylistAddError::APIError(error), err),
        };
        // Check the call history
        let calls = api.call_history.borrow();
        let expected = Some((playlist_name.to_string(), expected_tracks.to_vec()));
        let expected_track_id_call = Some(playlist_name.to_string());
        // Ensure that API was called correctly
        assert_eq!(expected, calls.add_tracks_to_playlist_called_with);
        assert_eq!(expected_track_id_call, calls.get_track_ids_in_playlist_called_with);
        // Ensure irrelevant function is not called
        assert_eq!(None, calls.create_playlist_called_with);
        assert_eq!(None, calls.get_playlist_id_called_with);
    }

    #[test]
    fn add_songs_to_playlist_filters_songs_already_in_playlist() {
        // Given
        let (playlist_name, adding_tracks, songs) = test_setup();
        // Two of the tracks are duplicates
        let existing_tracks = vec![adding_tracks[0].clone(), adding_tracks[2].clone()];
        // The expected result should be the single non-duplicate
        let expected_tracks = [adding_tracks[1].clone()];
        let api = MockPlaylistAPI::new(Ok(()), Ok(existing_tracks));
        // When
        // Ensure it doesn't fail using unwrap
        add_songs_to_playlist(&api, &playlist_name, songs).unwrap();
        // Then
        let expected = Some((playlist_name.to_string(), expected_tracks.to_vec()));
        let expected_track_id_call = Some(playlist_name.to_string());
        // Check the call history
        let calls = api.call_history.borrow();
        // Ensure that API was called correctly
        assert_eq!(expected, calls.add_tracks_to_playlist_called_with);
        assert_eq!(expected_track_id_call, calls.get_track_ids_in_playlist_called_with);
        // Ensure irrelevant function is not called
        assert_eq!(None, calls.create_playlist_called_with);
        assert_eq!(None, calls.get_playlist_id_called_with);
    }

    #[test]
    fn add_songs_to_playlist_get_track_ids_error() {
        // Given
        let (playlist_name, _, songs) = test_setup();
        let error = FakeError{};
        // Getting the track IDs will error
        let api = MockPlaylistAPI::new(Ok(()), Err(error));
        // When
        let result = add_songs_to_playlist(&api, &playlist_name, songs);
        // Then
        match result {
            // Ensure that we receive an error
            Ok(_) => assert!(false),
            // Make sure we receive the error
            Err(err) => assert_eq!(PlaylistAddError::APIError(error), err),
        };
        // Check the call history
        let calls = api.call_history.borrow();
        let expected_track_id_call = Some(playlist_name.to_string());
        // Ensure that API was called correctly
        assert_eq!(expected_track_id_call, calls.get_track_ids_in_playlist_called_with);
        // Ensure that we do not attempt to add when the earlier API call failed
        assert_eq!(None, calls.add_tracks_to_playlist_called_with);
        // Ensure irrelevant function is not called
        assert_eq!(None, calls.create_playlist_called_with);
        assert_eq!(None, calls.get_playlist_id_called_with);
    }

    #[test]
    fn add_songs_to_playlist_filters_duplicates() {
        // Given
        let (playlist_name, expected_tracks, mut songs) = test_setup();
        // Duplicate the second element
        let dupe = songs[1].clone();
        songs.push(dupe);
        let api = MockPlaylistAPI::new(Ok(()), Ok(Vec::new()));
        // When
        // Ensure it doesn't fail using unwrap
        add_songs_to_playlist(&api, &playlist_name, songs).unwrap();
        // Then
        let expected = Some((playlist_name.to_string(), expected_tracks.to_vec()));
        let expected_track_id_call = Some(playlist_name.to_string());
        // Check the call history
        let calls = api.call_history.borrow();
        // Ensure that API was called correctly
        assert_eq!(expected, calls.add_tracks_to_playlist_called_with);
        assert_eq!(expected_track_id_call, calls.get_track_ids_in_playlist_called_with);
        // Ensure irrelevant function is not called
        assert_eq!(None, calls.create_playlist_called_with);
        assert_eq!(None, calls.get_playlist_id_called_with);
    }

    #[test]
    fn add_songs_to_playlist_sends_error_when_no_new_tracks() {
        // Given
        let (playlist_name, adding_tracks, songs) = test_setup();
        // Two of the tracks are duplicates
        let existing_tracks = vec![
            adding_tracks[0].clone(),
            adding_tracks[1].clone(),
            adding_tracks[2].clone()
        ];
        let api = MockPlaylistAPI::new(Ok(()), Ok(existing_tracks));
        // When
        let result = add_songs_to_playlist(&api, &playlist_name, songs);
        match result {
            // Ensure that we receive an error
            Ok(_) => assert!(false),
            // Make sure we receive the error
            Err(err) => assert_eq!(PlaylistAddError::NoNewTracks(NoNewTracks{}), err),
        };
        // Then
        let expected_track_id_call = Some(playlist_name.to_string());
        // Check the call history
        let calls = api.call_history.borrow();
        // Ensure that API was called correctly
        assert_eq!(expected_track_id_call, calls.get_track_ids_in_playlist_called_with);
        assert_eq!(None, calls.add_tracks_to_playlist_called_with);
        // Ensure irrelevant function is not called
        assert_eq!(None, calls.create_playlist_called_with);
        assert_eq!(None, calls.get_playlist_id_called_with);
    }
}
