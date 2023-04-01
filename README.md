<h1 align="center" >
Motörhead
</h1>
<p align="center">
  <a href="https://twitter.com/Metal_io">
    <img src="https://img.shields.io/badge/metal-message?style=flat&logo=twitter&color=4f46e5&logoColor=#4f46e5" alt="Metal" style="margin-right:3px" />
  </a>
  <a href="https://discord.gg/GHY3Y8tU3J">
    <img src="https://dcbadge.vercel.app/api/server/GHY3Y8tU3J?compact=true&style=flat" alt="License" />
  </a>
  <a href="https://github.com/getmetal/motorhead/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/getmetal/motorhead?style=flat&label=license&logo=github&color=4f46e5&logoColor=fff" alt="License" />
  </a>
</p>

Motörhead is memory and information retrival server.

## Config

- `WINDOW_SIZE` - Number of max messages returned by the server. When this number is reached a job is triggered to halve it.
- `WINDOW_REDUCE_METHOD` - `summarization|buffer`
  - `summarization` - Once the `WINDOW_SIZE` is reached 1/4 of the window is summarized incrementally into the existing session summary.
  - `buffer`(default) - Memory only goes as far as the `WINDOW_SIZE`. Beyond that no messages are returned.
- `OPENAI_API_KEY` - Number of max messages returned by the server. When this number is reached a job is triggered to halve it. Required if `summarization` is the `WINDOW_REDUCE_METHOD`.

## Examples

- Check out our [Chat JS Example](examples/chat-js/)
