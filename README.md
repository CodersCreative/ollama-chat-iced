# OChat

#### A GUI made using iced and rust that allows you to talk to an AI.

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

## Run The App
### Install
```
# To install from crates.io
cargo install ochat

# To run the installed program
ochat
```

### Build & Run
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

## Gallery

![Ochat's home screen.](/media/home.png)

![Ochat handling images.](/media/images.png)

![Ochat's model options.](/media/models.png)

![Ochat's code handling.](/media/code.png)

![Ochat's theme showcase.](/media/options.png)

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)
