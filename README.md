# OChat [![Rust](https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white)]()

![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black) ![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)  ![macOS](https://img.shields.io/badge/mac%20os-000000?style=for-the-badge&logo=macos&logoColor=F0F0F0)

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
