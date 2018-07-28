# playlist-from-csv

A program used for adding songs to a playlist based on a CSV file.

This is to be used with [sir](https://github.com/oliveroneill/sir) which will
write to DynamoDB. You can then export the database to CSV and push the
song requests to the playlist using this program.

## Usage
```bash
cargo run <CLIENT_ID> <CLIENT_SECRET> <USERNAME> <PLAYLIST_NAME> <CSV_FILE_PATH>
```
