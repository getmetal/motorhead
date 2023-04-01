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

## Why use Motörhead?

When building chat applications using LLMs memory handling is something that is has to be built every time. Motörhead is a server to assist with that process, it provides 3 simple APIS:

- GET|POST|DELETE `/sessions/:id/memory`

A max `window_size` is set to for the LLM to keep track of the conversation. Once that max is hit Motörhead process the `window_size` / 2 messages and summarizes them. Subsequent summaries as the messages grow are incremental.

## Config

- `MAX_WINDOW_SIZE` (default:10) - Number of max messages returned by the server. When this number is reached a job is triggered to halve it.
- `OPENAI_API_KEY` (required)- Number of max messages returned by the server. When this number is reached a job is triggered to halve it.

## Examples

- Check out our [Chat JS Example](examples/chat-js/)
