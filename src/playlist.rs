use std::error::Error;
use std::fmt;

/// A trait for querying for playlists
pub trait PlaylistAPI<E> {
    /// Get the ID for the playlist name for later queries to the API
    ///
    /// # Arguments
    ///
    /// * `playlist_name` - A string slice that holds the playlist name
    fn get_playlist_id(&self, playlist_name: &str) -> Result<String, PlaylistError<E>>;
    /// Create a playlist with a given name
    ///
    /// # Arguments
    ///
    /// * `playlist_name` - A string slice that holds the playlist name
    fn create_playlist(&self, playlist_name: &str) -> Result<String, E>;
}

/// Playlist enum for different playlist errors
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlaylistError<E> {
    /// Generic playlist error from API
    APIError(E),
    /// The error when the playlist cannot be found
    PlaylistNotFound(PlaylistNotFound),
}

/// An error when the playlist name is not found
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PlaylistNotFound {}

impl Error for PlaylistNotFound {
    fn description(&self) -> &str {
        "Could not find playlist"
    }
}

impl fmt::Display for PlaylistNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Get the playlist ID for the given name and if it doesn't exist then create
/// a playlist with the given name.
///
/// # Arguments
///
/// * `playlist_name` - A string slice that holds the playlist name
pub fn get_playlist_id_create_if_needed<E>(api: &PlaylistAPI<E>, playlist_name: &str) -> Result<String, PlaylistError<E>> {
    match api.get_playlist_id(playlist_name) {
        Ok(playlist_id) => Ok(playlist_id),
        Err(error) => {
            match error {
                PlaylistError::PlaylistNotFound(_) => {
                    let id = api.create_playlist(playlist_name).map_err(PlaylistError::APIError)?;
                    Ok(id)
                },
                PlaylistError::APIError(e) => Err(PlaylistError::APIError(e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Keep track of calls made to MockPlaylistAPI
    #[derive(Debug, Clone)]
    struct CallHistory {
        create_playlist_called_with: Option<String>,
        get_playlist_id_called_with: Option<String>,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct FakeError {}

    /// A fake API where you can specify the return values to the functions
    struct MockPlaylistAPI {
        call_history: RefCell<CallHistory>,
        get_playlist_id_returns: Result<String, PlaylistError<FakeError>>,
        create_playlist_returns: Result<String, FakeError>,
    }

    impl MockPlaylistAPI {
        /// Create a new MockPlaylistAPI
        fn new(get_playlist_id_returns: Result<String, PlaylistError<FakeError>>,
            create_playlist_returns: Result<String, FakeError>) -> MockPlaylistAPI {
            MockPlaylistAPI {
                call_history: RefCell::new(
                    CallHistory{
                        create_playlist_called_with: None,
                        get_playlist_id_called_with: None,
                    }
                ),
                get_playlist_id_returns: get_playlist_id_returns,
                create_playlist_returns: create_playlist_returns,
            }
        }
    }

    impl PlaylistAPI<FakeError> for MockPlaylistAPI {
        fn get_playlist_id(&self, playlist_name: &str) -> Result<String, PlaylistError<FakeError>> {
            self.call_history.borrow_mut().get_playlist_id_called_with = Some(playlist_name.to_owned());
            self.get_playlist_id_returns.clone()
        }

        fn create_playlist(&self, playlist_name: &str) -> Result<String, FakeError> {
            self.call_history.borrow_mut().create_playlist_called_with = Some(playlist_name.to_owned());
            self.create_playlist_returns.clone()
        }
    }

    #[test]
    fn get_playlist_id_if_playlist_exists() {
        // Given
        let playlist_name = "test_playlist_name1";
        let expected_playlist_id = "id_123";
        let api = MockPlaylistAPI::new(
            Ok(expected_playlist_id.to_owned()),
            Ok(expected_playlist_id.to_owned())
        );
        // When
        let result = get_playlist_id_create_if_needed(&api, playlist_name).unwrap();
        // Then
        assert_eq!(expected_playlist_id, result);
        // Ensure that API was called correctly
        assert_eq!(Some(playlist_name.to_owned()), api.call_history.borrow().get_playlist_id_called_with);
        // Ensure that the create call is not made
        assert_eq!(None, api.call_history.borrow().create_playlist_called_with);
    }

    #[test]
    fn get_playlist_id_create_if_needed_creates_playlist() {
        // Given
        let playlist_name = "test_playlist_name1";
        let expected_playlist_id = "id_123";
        // The get call will fail with the playlist not being found
        let get_error = PlaylistError::PlaylistNotFound(PlaylistNotFound{});
        let api = MockPlaylistAPI::new(
            // The get call will fail
            Err(get_error),
            Ok(expected_playlist_id.to_owned()),
        );
        // When
        let result = get_playlist_id_create_if_needed(&api, playlist_name).unwrap();
        // Then
        assert_eq!(expected_playlist_id, result);
        assert_eq!(Some(playlist_name.to_owned()), api.call_history.borrow().get_playlist_id_called_with);
        // Ensure that the create call is made since the playlist was not found
        assert_eq!(Some(playlist_name.to_owned()), api.call_history.borrow().create_playlist_called_with);
    }

    #[test]
    fn get_playlist_id_handles_api_error() {
        // Given
        let playlist_name = "test_playlist_name1";
        let expected_playlist_id = "id_123";
        // The get call will fail with an API error
        let get_error = PlaylistError::APIError(FakeError{});
        let api = MockPlaylistAPI::new(
            Err(get_error),
            Ok(expected_playlist_id.to_owned())
        );
        // When
        let result = get_playlist_id_create_if_needed(&api, playlist_name);
        // Then
        match result {
            // Fail if it doesn't send back an error
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(get_error, err),
        };
        assert_eq!(Some(playlist_name.to_owned()), api.call_history.borrow().get_playlist_id_called_with);
        // Ensure that we do not create a playlist since an error occurred
        assert_eq!(None, api.call_history.borrow().create_playlist_called_with);
    }

        #[test]
    fn get_playlist_id_fails_on_create() {
        // Given
        let playlist_name = "test_playlist_name1";
        let create_error = FakeError{};
        let api = MockPlaylistAPI::new(
            // The get call will fail with the playlist not found
            Err(PlaylistError::PlaylistNotFound(PlaylistNotFound{})),
            Err(create_error),
        );
        // When
        let result = get_playlist_id_create_if_needed(&api, playlist_name);
        // Then
        match result {
            // Ensure that we receive an error
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(PlaylistError::APIError(create_error), err),
        };
        assert_eq!(Some(playlist_name.to_owned()), api.call_history.borrow().get_playlist_id_called_with);
        assert_eq!(Some(playlist_name.to_owned()), api.call_history.borrow().create_playlist_called_with);
    }
}
