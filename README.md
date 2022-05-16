# Orga Whereabouts


## Usage

```sh
$ orga-whereabouts -c config.toml -i /dev/input/event23
```


## Sound Formats

Ogg Vorbis and MP3 are supported out of the box. However, the employed
audio playback library ([rodio](https://github.com/RustAudio/rodio))
also supports FLAC, WAV, MP4 and AAC, but those have to be enabled as
features in `Cargo.toml` and require recompilation of the program.


## License

Orga Whereabouts is licensed under the MIT license.


## Author

Orga Whereabouts was created by Jochen Kupperschmidt.
