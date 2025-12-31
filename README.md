# OChat [![Rust](https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white)]()

![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black) ![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)  ![macOS](https://img.shields.io/badge/mac%20os-000000?style=for-the-badge&logo=macos&logoColor=F0F0F0)

#### A GUI made using iced and rust that allows you to easily talk to AI models.

## Key Features

- 🚀 **Effortless Setup**: Install seamlessly using [Cargo](https://doc.rust-lang.org/cargo/).

- 🤝 **Ollama**: Effortlessly chat to any AI model available at [ollama](https://ollama.com/search) and download each model within the app.

- 🤝 **Huggingface**: Allows for models to be run and downloaded that are available at [huggingface](https://huggingface.co).

- ⚙️ **Model Settings**: Easily change the parameters of the model within the application.

- 🔢 **Full Markdown Support**: Elevate your LLM experience with comprehensive Markdown support for enriched interaction and styling.

- 🎤 **Voice Calls**: Experience seamless communication with integrated voice call features, allowing for a more dynamic and interactive chat environment.

- 🎤 **Transcribe**: Easily transcribe mic input within the application for hands-free use.

- 🗔 **Panels**: Engage with multiple activities within the app simultaneously using distinct panels or split panels into new seperate windows.

- ⚙️ **Many Models Conversations**: Effortlessly engage with various models simultaneously, harnessing their unique strengths for optimal responses. Enhance your experience by leveraging a diverse set of models in parallel.

- 🌟 **Continuous Updates**: I am committed to improving ochat with regular updates, fixes, and new features.

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

# To install additional binaries for the gui and server
ochat install

# To run the installed program
ochat
```

## Gallery

![Ochat's home screen.](/media/images.png)

![Ochat calling screen.](/media/call.png)

![Ochat chats split.](/media/split_chats.png)

![Ochat's model options.](/media/gen_options.png)

![Ochat's code handling.](/media/code.png)

![Ochat's theme showcase.](/media/split.png)

![Ochat pulling models.](/media/pulling.png)

![Ochat over multiple windows.](/media/multi_window.png)

![Ochat prompts.](/media/prompts.png)

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)

