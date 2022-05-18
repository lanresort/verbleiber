# Verbleiber


## Usage

```sh
$ verbleiber -c config.toml -i /dev/input/event23
```


## Sound Formats

Ogg Vorbis is supported out of the box. However, the employed audio
playback library ([rodio](https://github.com/RustAudio/rodio)) also
supports MP3, FLAC, WAV, MP4 and AAC, but those have to be enabled as
features in `Cargo.toml` and require recompilation of the program.


## License

Verbleiber is licensed under the MIT license.


## Author

Verbleiber was created by Jochen Kupperschmidt.
