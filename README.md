# SADE Voice Assistant Rust

#### SADE is a rust voice assistant which is was made to be easy to set up, customize and learn from.

## Installation
### Download LLM Model

[Install Ollama](https://ollama.ai/download)\
[Pull Orca-Mini](https://ollama.ai/library/orca-mini:3b)

On Linux:
```
# Install ollama:
curl https://ollama.ai/install.sh | sh

# Pull orca-mini:
ollama pull orca-mini:3b
```

### Install Rust

[Install Rust](https://www.rust-lang.org/tools/install)

On Linux or MacOS:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Run SADE Voice Assistant

```
# Clone the repository:
git clone https://gitlab.com/officialccoders/ollama-chat-iced.git
cd ollama-chat-iced

# Build and run app with release tags:
cargo build --release
cargo run --release

# Or simply:
cargo run
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)
